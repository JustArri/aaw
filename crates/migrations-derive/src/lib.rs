use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::{ToTokens, format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, Error, Expr, ExprLit, Fields, Lit, Meta, Token, Type,
    parse_macro_input, punctuated::Punctuated,
};

#[proc_macro_derive(Migrations, attributes(migrations, schema, serde))]
pub fn derive_migrations(input: TokenStream) -> TokenStream {
    expand_migrations(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn expand_migrations(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let (version, version_type) = parse_configuration(&input)?;
    if version != 1 {
        return Err(Error::new_spanned(
            &input.ident,
            "only the initial migration version is currently supported",
        ));
    }

    if !input.generics.params.is_empty() || input.generics.where_clause.is_some() {
        return Err(Error::new_spanned(
            &input.generics,
            "Migrations does not support generic structs",
        ));
    }

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(Error::new_spanned(
                    &data.fields,
                    "Migrations requires a struct with named fields",
                ));
            }
        },
        _ => {
            return Err(Error::new_spanned(
                &input.ident,
                "Migrations can only be derived for structs",
            ));
        }
    };

    if let Some(field) =
        fields.iter().find(|field| field.ident.as_ref().is_some_and(|ident| ident == "version"))
    {
        return Err(Error::new_spanned(
            field,
            "`version` is generated as wire metadata; remove it from the input struct",
        ));
    }

    let version_literal = Literal::u32_unsuffixed(version);
    let name = &input.ident;
    let visibility = &input.vis;
    let preview = format_ident!("{name}Preview");
    let versioned = format_ident!("{name}Versioned");
    let current_wire = format_ident!("{name}V{version}");
    let current_variant = format_ident!("V{version}");
    let serialize_wire = format_ident!("__{name}Serialize");
    let schema_wire = format_ident!("__{name}Schema");

    let field_names: Vec<_> = fields
        .iter()
        .map(|field| field.ident.as_ref().expect("named fields were checked"))
        .collect();
    let wire_field_definitions = fields.iter().map(|field| {
        let attrs = serde_attrs(&field.attrs);
        let visibility = &field.vis;
        let ident = field.ident.as_ref().expect("named fields were checked");
        let ty = &field.ty;
        quote! {
            #(#attrs)*
            #visibility #ident: #ty
        }
    });
    let serialize_field_definitions = fields.iter().map(|field| {
        let attrs = serde_attrs(&field.attrs);
        let ident = field.ident.as_ref().expect("named fields were checked");
        let ty = &field.ty;
        quote! {
            #(#attrs)*
            #ident: &'__migrations #ty
        }
    });
    let schema_field_definitions = fields.iter().map(|field| {
        let attrs = field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("serde") || attr.path().is_ident("schema"));
        let ident = field.ident.as_ref().expect("named fields were checked");
        let ty = &field.ty;
        quote! {
            #(#attrs)*
            #ident: #ty
        }
    });

    let serde_container_attrs: Vec<_> = serde_attrs(&input.attrs).collect();
    let schema_attrs: Vec<_> = input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("schema"))
        .map(|attr| quote!(#attr))
        .collect();
    let schema_attrs =
        if schema_attrs.is_empty() { vec![quote!(#[schema(as = #name)])] } else { schema_attrs };
    let error_context = name.to_string();

    Ok(quote! {
        #[derive(::serde::Deserialize)]
        struct #preview {
            version: #version_type,
        }

        #visibility enum #versioned {
            #current_variant(#current_wire),
            Current(#name),
        }

        impl #versioned {
            pub const CURRENT_VERSION: #version_type = #version_literal;

            pub fn version(&self) -> ::core::option::Option<#version_type> {
                match self {
                    Self::#current_variant(_) => Some(#version_literal),
                    Self::Current(_) => None,
                }
            }

            pub fn next_version(&self) -> ::core::option::Option<#version_type> {
                let version = self.version()?;
                if version < Self::CURRENT_VERSION { Some(version + 1) } else { None }
            }

            pub fn migrate_to_next(self) -> Self {
                match self {
                    Self::#current_variant(value) => Self::Current(#name {
                        #(#field_names: value.#field_names,)*
                    }),
                    current @ Self::Current(_) => current,
                }
            }

            pub fn migrate(self) -> #name {
                let mut versioned = self;

                loop {
                    if let Self::Current(value) = versioned {
                        return value;
                    }
                    versioned = versioned.migrate_to_next();
                }
            }
        }

        impl<'de> ::serde::Deserialize<'de> for #versioned {
            fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                use ::serde::de::Error as _;

                let value = <::serde_json::Value as ::serde::Deserialize>::deserialize(deserializer)?;
                let preview = <#preview as ::serde::Deserialize>::deserialize(&value)
                    .map_err(D::Error::custom)?;

                match preview.version {
                    #version_literal => {
                        let value = ::serde_json::from_value::<#current_wire>(value)
                            .map_err(D::Error::custom)?;
                        Ok(Self::#current_variant(value))
                    }
                    version => Err(D::Error::custom(format_args!(
                        "unsupported version: {version} ({})",
                        #error_context,
                    ))),
                }
            }
        }

        #[derive(::serde::Deserialize)]
        #(#serde_container_attrs)*
        #visibility struct #current_wire {
            #(#wire_field_definitions,)*
        }

        impl<'de> ::serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> ::core::result::Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                let versioned = <#versioned as ::serde::Deserialize>::deserialize(deserializer)?;
                Ok(versioned.migrate())
            }
        }

        #[derive(::serde::Serialize)]
        #(#serde_container_attrs)*
        struct #serialize_wire<'__migrations> {
            version: #version_type,
            #(#serialize_field_definitions,)*
        }

        impl ::serde::Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                let wire = #serialize_wire {
                    version: #versioned::CURRENT_VERSION,
                    #(#field_names: &self.#field_names,)*
                };
                ::serde::Serialize::serialize(&wire, serializer)
            }
        }

        #[derive(::utoipa::ToSchema)]
        #(#schema_attrs)*
        #(#serde_container_attrs)*
        struct #schema_wire {
            version: #version_type,
            #(#schema_field_definitions,)*
        }

        impl ::utoipa::PartialSchema for #name {
            fn schema() -> ::utoipa::openapi::RefOr<::utoipa::openapi::schema::Schema> {
                <#schema_wire as ::utoipa::PartialSchema>::schema()
            }
        }

        impl ::utoipa::ToSchema for #name {
            fn name() -> ::std::borrow::Cow<'static, str> {
                <#schema_wire as ::utoipa::ToSchema>::name()
            }

            fn schemas(
                schemas: &mut ::std::vec::Vec<(
                    ::std::string::String,
                    ::utoipa::openapi::RefOr<::utoipa::openapi::schema::Schema>,
                )>,
            ) {
                <#schema_wire as ::utoipa::ToSchema>::schemas(schemas)
            }
        }
    })
}

fn serde_attrs(attrs: &[Attribute]) -> impl Iterator<Item = &Attribute> {
    attrs.iter().filter(|attr| attr.path().is_ident("serde"))
}

fn parse_configuration(input: &DeriveInput) -> syn::Result<(u32, Type)> {
    let attribute =
        input.attrs.iter().find(|attribute| attribute.path().is_ident("migrations")).ok_or_else(
            || {
                Error::new_spanned(
                    &input.ident,
                    "missing `#[migrations(version = N, version_type = u32)]` attribute",
                )
            },
        )?;

    let arguments = attribute.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
    let mut version = None;
    let mut version_type = None;

    for argument in arguments {
        let Meta::NameValue(name_value) = argument else {
            return Err(Error::new_spanned(argument, "expected a name-value argument"));
        };

        if name_value.path.is_ident("version") {
            let Expr::Lit(ExprLit { lit: Lit::Int(value), .. }) = name_value.value else {
                return Err(Error::new_spanned(name_value.value, "version must be an integer"));
            };
            let parsed = value.base10_parse::<u32>()?;
            if parsed == 0 {
                return Err(Error::new_spanned(value, "migration versions start at 1"));
            }
            version = Some(parsed);
        } else if name_value.path.is_ident("version_type") {
            let ty = syn::parse2::<Type>(name_value.value.into_token_stream())?;
            validate_version_type(&ty)?;
            version_type = Some(ty);
        } else {
            return Err(Error::new_spanned(name_value.path, "unknown migrations argument"));
        }
    }

    let version = version.ok_or_else(|| Error::new_spanned(attribute, "missing `version`"))?;
    let version_type =
        version_type.ok_or_else(|| Error::new_spanned(attribute, "missing `version_type`"))?;
    Ok((version, version_type))
}

fn validate_version_type(ty: &Type) -> syn::Result<()> {
    let valid = match ty {
        Type::Path(path) if path.qself.is_none() => path.path.get_ident().is_some_and(|ident| {
            matches!(ident.to_string().as_str(), "u8" | "u16" | "u32" | "u64" | "u128" | "usize")
        }),
        _ => false,
    };

    if valid {
        Ok(())
    } else {
        Err(Error::new_spanned(ty, "version_type must be an unsigned integer type"))
    }
}
