use std::collections::{BTreeMap, HashMap};

pub trait StableIndex: Copy + From<usize> + Into<usize> {}

impl<I> StableIndex for I where I: Copy + From<usize> + Into<usize> {}

#[derive(Debug, Clone)]
pub struct Siv<T, I = usize>
where
    I: StableIndex,
{
    indices: Vec<usize>,
    ids: Vec<I>,
    values: Vec<T>,
}

impl<T, I> Siv<T, I>
where
    I: StableIndex,
{
    fn from_entries(entries: impl IntoIterator<Item = (I, T)>) -> Self {
        let (mut ids, values): (Vec<_>, Vec<_>) = entries.into_iter().unzip();
        let Some(max_id) = ids.iter().map(|id| (*id).into()).max() else {
            return Self::new();
        };

        let mut indices = vec![usize::MAX; max_id + 1];
        for (index, id) in ids.iter().enumerate() {
            indices[(*id).into()] = index;
        }

        for (id, index) in indices.iter_mut().enumerate() {
            if *index == usize::MAX {
                *index = ids.len();
                ids.push(I::from(id));
            }
        }

        Self { indices, ids, values }
    }

    pub const fn new() -> Self {
        Self { indices: Vec::new(), ids: Vec::new(), values: Vec::new() }
    }

    pub fn clone_ids<U>(&self) -> Siv<U, I> {
        Siv {
            indices: self.indices.clone(),
            ids: self.ids.clone(),
            values: Vec::with_capacity(self.values.capacity()),
        }
    }

    pub fn map_ref<U, F>(&self, f: F) -> Siv<U, I>
    where
        F: FnMut(&T) -> U,
    {
        Siv {
            indices: self.indices.clone(),
            ids: self.ids.clone(),
            values: self.values.iter().map(f).collect(),
        }
    }

    pub fn map<U, F>(self, f: F) -> Siv<U, I>
    where
        F: FnMut(T) -> U,
    {
        Siv {
            indices: self.indices,
            ids: self.ids,
            values: self.values.into_iter().map(f).collect(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            indices: Vec::with_capacity(capacity),
            ids: Vec::with_capacity(capacity),
            values: Vec::with_capacity(capacity),
        }
    }

    pub fn ensure_capacity(&mut self, target_capacity: usize) {
        if target_capacity <= self.capacity() {
            return;
        }

        self.indices.reserve(target_capacity - self.indices.len());
        self.ids.reserve(target_capacity - self.ids.len());
        self.values.reserve(target_capacity - self.values.len());
    }

    pub fn push(&mut self, value: T) -> I {
        let di_len = self.indices.len();
        let d_len = self.values.len();

        if di_len == d_len {
            self.indices.push(di_len);
            self.ids.push(I::from(di_len));
        }

        self.values.push(value);
        self.ids[d_len]
    }

    pub const fn capacity(&self) -> usize {
        self.values.capacity()
    }

    pub fn remove(&mut self, id: I) -> T {
        let raw_id = id.into();
        match self.try_remove(id) {
            Some(elem) => elem,
            None => panic!("removal id (is {raw_id}) is not valid"),
        }
    }

    pub fn try_remove(&mut self, id: I) -> Option<T> {
        let id = id.into();
        let index = *self.indices.get(id)?;
        let d_len = self.values.len();

        if index >= d_len {
            return None;
        }

        let last_index = d_len - 1;
        let moved_id = (*self.ids.get(last_index)?).into();

        self.indices.swap(id, moved_id);
        self.ids.swap(index, last_index);
        Some(self.values.swap_remove(index))
    }

    pub fn replace(&mut self, id: I, value: T) -> T {
        let raw_id = id.into();
        match self.try_replace(id, value) {
            Ok(previous) => previous,
            Err(_) => panic!("replacement id (is {raw_id}) is not valid"),
        }
    }

    pub fn try_replace(&mut self, id: I, value: T) -> Result<T, T> {
        let id = id.into();
        let Some(index) = self.indices.get(id).copied() else {
            return Err(value);
        };
        let Some(current) = self.values.get_mut(index) else {
            return Err(value);
        };
        Ok(std::mem::replace(current, value))
    }

    pub fn clear(&mut self) {
        self.indices.clear();
        self.ids.clear();
        self.values.clear();
    }

    pub const fn len(&self) -> usize {
        self.values.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn get(&self, id: I) -> Option<&T> {
        let id = id.into();
        let index = *self.indices.get(id)?;
        self.values.get(index)
    }

    pub fn get_mut(&mut self, id: I) -> Option<&mut T> {
        let id = id.into();
        let index = *self.indices.get(id)?;
        self.values.get_mut(index)
    }

    pub fn iter_values(&self) -> std::slice::Iter<'_, T> {
        self.values.iter()
    }

    pub fn iter_values_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.values.iter_mut()
    }

    pub fn iter_ids(&self) -> std::slice::Iter<'_, I> {
        self.ids.iter()
    }

    pub fn indices(&self) -> &[usize] {
        &self.indices
    }

    pub fn ids(&self) -> &[I] {
        &self.ids
    }

    pub fn values(&self) -> &[T] {
        &self.values
    }
}

impl<T, E, I> Siv<Result<T, E>, I>
where
    I: StableIndex,
{
    pub fn try_collect_ref<F>(&self) -> Result<Siv<T, I>, F>
    where
        T: Clone,
        E: Clone,
        F: From<E>,
    {
        Ok(Siv {
            indices: self.indices.clone(),
            ids: self.ids.clone(),
            values: self
                .values
                .iter()
                .map(|result| result.clone().map_err(F::from))
                .collect::<Result<_, F>>()?,
        })
    }

    pub fn try_collect<F>(self) -> Result<Siv<T, I>, F>
    where
        F: From<E>,
    {
        Ok(Siv {
            indices: self.indices,
            ids: self.ids,
            values: self
                .values
                .into_iter()
                .map(|result| result.map_err(F::from))
                .collect::<Result<_, F>>()?,
        })
    }
}

impl<T, I> Default for Siv<T, I>
where
    I: StableIndex,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, I> From<Vec<T>> for Siv<T, I>
where
    I: StableIndex,
{
    fn from(values: Vec<T>) -> Self {
        let mut indices = Vec::with_capacity(values.capacity());
        let mut ids = Vec::with_capacity(values.capacity());

        for i in 0..values.len() {
            indices.push(i);
            ids.push(I::from(i));
        }

        Self { indices, ids, values }
    }
}

impl<T, I> From<HashMap<I, T>> for Siv<T, I>
where
    I: StableIndex + Eq + std::hash::Hash,
{
    fn from(value: HashMap<I, T>) -> Self {
        Self::from_entries(value)
    }
}

impl<T, I> From<BTreeMap<I, T>> for Siv<T, I>
where
    I: StableIndex + Ord,
{
    fn from(value: BTreeMap<I, T>) -> Self {
        Self::from_entries(value)
    }
}

impl<T, I> From<Siv<T, I>> for HashMap<I, T>
where
    I: StableIndex + Eq + std::hash::Hash,
{
    fn from(value: Siv<T, I>) -> Self {
        value.into_iter().collect()
    }
}

impl<'a, T, I> IntoIterator for &'a Siv<T, I>
where
    I: StableIndex,
{
    type Item = (&'a I, &'a T);
    type IntoIter = std::iter::Zip<std::slice::Iter<'a, I>, std::slice::Iter<'a, T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.ids.iter().zip(self.values.iter())
    }
}

impl<'a, T, I> IntoIterator for &'a mut Siv<T, I>
where
    I: StableIndex,
{
    type Item = (&'a I, &'a mut T);
    type IntoIter = std::iter::Zip<std::slice::Iter<'a, I>, std::slice::IterMut<'a, T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.ids.iter().zip(self.values.iter_mut())
    }
}

impl<T, I> IntoIterator for Siv<T, I>
where
    I: StableIndex,
{
    type Item = (I, T);
    type IntoIter = std::iter::Zip<std::vec::IntoIter<I>, std::vec::IntoIter<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.ids.into_iter().zip(self.values)
    }
}

impl<T, I> serde::Serialize for Siv<T, I>
where
    T: serde::Serialize,
    I: StableIndex + serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(Some(self.values.len()))?;

        for i in 0..self.values.len() {
            map.serialize_entry(&self.ids[i], &self.values[i])?;
        }

        map.end()
    }
}

impl<'de, T, I> serde::Deserialize<'de> for Siv<T, I>
where
    T: serde::Deserialize<'de>,
    I: StableIndex + serde::Deserialize<'de> + Ord,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map = BTreeMap::<I, T>::deserialize(deserializer)?;
        Ok(map.into())
    }
}

impl<T, I> utoipa::__dev::ComposeSchema for Siv<T, I>
where
    T: utoipa::__dev::ComposeSchema,
    I: StableIndex + utoipa::__dev::ComposeSchema,
{
    fn compose(
        schemas: Vec<utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>>,
    ) -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        <BTreeMap<I, T> as utoipa::__dev::ComposeSchema>::compose(schemas)
    }
}

impl<T, I> utoipa::ToSchema for Siv<T, I>
where
    T: utoipa::ToSchema + utoipa::__dev::ComposeSchema,
    I: StableIndex + utoipa::ToSchema + utoipa::__dev::ComposeSchema,
{
    fn schemas(
        schemas: &mut Vec<(String, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>)>,
    ) {
        I::schemas(schemas);
        T::schemas(schemas);
    }
}

#[cfg(test)]
mod tests {
    use super::Siv;
    use std::collections::{BTreeMap, HashMap};
    use utoipa::PartialSchema;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, utoipa::ToSchema)]
    struct TestId(usize);

    impl From<usize> for TestId {
        fn from(id: usize) -> Self {
            Self(id)
        }
    }

    impl From<TestId> for usize {
        fn from(id: TestId) -> Self {
            id.0
        }
    }

    type Sources<T> = Siv<T, TestId>;

    #[test]
    fn allocates_strongly_typed_ids_and_supports_mutation() {
        let mut values = Sources::from(vec!["first", "second"]);
        let second = TestId(1);

        assert_eq!(values.push("third"), TestId(2));
        assert_eq!(values.get(second), Some(&"second"));
        *values.get_mut(second).unwrap() = "changed";
        assert_eq!(values.remove(second), "changed");
        assert!(values.try_remove(second).is_none());
    }

    #[test]
    fn removing_an_element_preserves_other_ids_and_reuses_the_removed_id() {
        let mut values = Sources::from(vec!["zero", "one", "two", "three"]);

        assert_eq!(values.remove(TestId(1)), "one");

        assert_eq!(values.get(TestId(0)), Some(&"zero"));
        assert_eq!(values.get(TestId(1)), None);
        assert_eq!(values.get(TestId(2)), Some(&"two"));
        assert_eq!(values.get(TestId(3)), Some(&"three"));

        let replacement = values.push("replacement");
        assert_eq!(replacement, TestId(1));
        assert_eq!(values.get(replacement), Some(&"replacement"));
    }

    #[test]
    fn replacing_an_element_preserves_its_id() {
        let mut values = Sources::from(vec!["zero", "one", "two", "three"]);
        values.remove(TestId(1));

        assert_eq!(values.replace(TestId(3), "new three"), "three");
        assert_eq!(values.get(TestId(3)), Some(&"new three"));
        assert_eq!(values.get(TestId(1)), None);
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn try_replace_rejects_invalid_and_removed_ids_without_changing_values() {
        let mut values = Sources::from(vec!["zero", "one", "two"]);
        values.remove(TestId(1));

        assert_eq!(
            values.try_replace(TestId(1), "removed replacement"),
            Err("removed replacement")
        );
        assert_eq!(
            values.try_replace(TestId(8), "invalid replacement"),
            Err("invalid replacement")
        );
        assert_eq!(values.get(TestId(0)), Some(&"zero"));
        assert_eq!(values.get(TestId(1)), None);
        assert_eq!(values.get(TestId(2)), Some(&"two"));
    }

    #[test]
    fn invalid_ids_are_rejected_without_changing_the_vector() {
        let mut values = Sources::from(vec![10, 20]);

        assert_eq!(values.get(TestId(8)), None);
        assert_eq!(values.get_mut(TestId(8)), None);
        assert_eq!(values.try_remove(TestId(8)), None);
        assert_eq!(values.values(), &[10, 20]);
    }

    #[test]
    #[should_panic(expected = "removal id (is 8) is not valid")]
    fn remove_panics_for_an_invalid_id() {
        Sources::from(vec![10]).remove(TestId(8));
    }

    #[test]
    #[should_panic(expected = "replacement id (is 8) is not valid")]
    fn replace_panics_for_an_invalid_id() {
        Sources::from(vec![10]).replace(TestId(8), 80);
    }

    #[test]
    fn iteration_keeps_each_id_paired_with_its_value() {
        let mut values = Sources::from(vec!["zero", "one", "two"]);
        values.remove(TestId(1));

        let pairs: HashMap<_, _> = (&values).into_iter().map(|(id, value)| (*id, *value)).collect();

        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[&TestId(0)], "zero");
        assert_eq!(pairs[&TestId(2)], "two");
    }

    #[test]
    fn mapping_values_preserves_stable_ids() {
        let mut values = Sources::from(vec![1, 2, 3]);
        values.remove(TestId(1));

        let mapped = values.map_ref(|value| value * 10);

        assert_eq!(mapped.get(TestId(0)), Some(&10));
        assert_eq!(mapped.get(TestId(1)), None);
        assert_eq!(mapped.get(TestId(2)), Some(&30));
    }

    #[test]
    fn collecting_results_preserves_ids_and_propagates_errors() {
        let mut successful = Sources::from(vec![Ok::<_, &str>(10), Ok(20), Ok(30)]);
        assert_eq!(successful.remove(TestId(1)), Ok(20));

        let collected = successful.try_collect::<&str>().unwrap();
        assert_eq!(collected.get(TestId(0)), Some(&10));
        assert_eq!(collected.get(TestId(1)), None);
        assert_eq!(collected.get(TestId(2)), Some(&30));

        let failed = Sources::from(vec![Ok(10), Err("broken")]);
        assert_eq!(failed.try_collect::<&str>().unwrap_err(), "broken");
    }

    #[test]
    fn sparse_hash_map_conversion_keeps_holes_invalid_and_reusable() {
        let map = HashMap::from([(TestId(1), "one"), (TestId(3), "three")]);
        let mut values = Sources::from(map);

        assert_eq!(values.get(TestId(0)), None);
        assert_eq!(values.get(TestId(1)), Some(&"one"));
        assert_eq!(values.get(TestId(2)), None);
        assert_eq!(values.get(TestId(3)), Some(&"three"));

        let reused = values.push("reused");
        assert!(matches!(reused, TestId(0) | TestId(2)));
        assert_eq!(values.get(reused), Some(&"reused"));
    }

    #[test]
    fn serde_round_trip_preserves_holes_and_restores_id_order() {
        let mut values = Siv::from(vec!["zero", "one", "two"]);
        values.remove(0);

        let json = serde_json::to_string(&values).unwrap();
        let mut restored: Siv<&str> = serde_json::from_str(&json).unwrap();
        let pairs: Vec<_> = (&restored).into_iter().map(|(id, value)| (*id, *value)).collect();

        assert_eq!(pairs, vec![(1, "one"), (2, "two")]);
        assert_eq!(restored.get(0), None);
        assert_eq!(restored.get(1), Some(&"one"));
        assert_eq!(restored.get(2), Some(&"two"));
        assert_eq!(restored.push("replacement"), 0);
    }

    #[test]
    fn schema_matches_the_serialized_map_representation() {
        assert!(Sources::<String>::schema() == BTreeMap::<TestId, String>::schema());
    }

    #[test]
    fn clear_resets_values_and_id_allocation() {
        let mut values = Sources::from(vec!["zero", "one"]);
        values.clear();

        assert!(values.is_empty());
        assert!(values.ids().is_empty());
        assert!(values.indices().is_empty());
        assert_eq!(values.push("new zero"), TestId(0));
    }

    #[test]
    fn capacity_can_be_reserved_without_changing_contents() {
        let mut values = Sources::<i32>::with_capacity(2);
        assert!(values.capacity() >= 2);

        values.push(10);
        values.ensure_capacity(16);

        assert!(values.capacity() >= 16);
        assert_eq!(values.len(), 1);
        assert_eq!(values.get(TestId(0)), Some(&10));
    }
}
