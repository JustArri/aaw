pub async fn poweroff() -> zbus::Result<()> {
    let connection = zbus::Connection::system().await?;

    let proxy = zbus::Proxy::new(
        &connection,
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        "org.freedesktop.login1.Manager",
    )
    .await?;

    proxy.call_method("PowerOff", &(false,)).await?;

    Ok(())
}
