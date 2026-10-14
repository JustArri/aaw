pub async fn poweroff() -> zbus::Result<()> {
    call_login1("PowerOff").await
}

pub async fn reboot() -> zbus::Result<()> {
    call_login1("Reboot").await
}

async fn call_login1(method: &'static str) -> zbus::Result<()> {
    let connection = zbus::Connection::system().await?;

    let proxy = zbus::Proxy::new(
        &connection,
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        "org.freedesktop.login1.Manager",
    )
    .await?;

    proxy.call_method(method, &(false,)).await?;

    Ok(())
}
