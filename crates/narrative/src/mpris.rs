use std::collections::HashMap;
use std::ops::Deref;
use zbus::zvariant::{OwnedValue, Value};
use zbus::Connection;

#[zbus::proxy(
    interface = "org.mpris.MediaPlayer2.Player",
    default_path = "/org/mpris/MediaPlayer2"
)]
trait MediaPlayer2Player {
    fn play_pause(&self) -> zbus::Result<()>;
    fn next(&self) -> zbus::Result<()>;
    fn previous(&self) -> zbus::Result<()>;

    #[zbus(property)]
    fn playback_status(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn metadata(&self) -> zbus::Result<HashMap<String, OwnedValue>>;

    #[zbus(property)]
    fn position(&self) -> zbus::Result<i64>;
}

#[derive(Debug, Clone)]
pub struct MediaInfo {
    pub title: String,
    pub artist: String,
    pub progress_secs: u32,
    pub duration_secs: u32,
    pub playing: bool,
    pub service: String,
}

#[derive(Debug, Clone)]
pub enum MediaCommand {
    PlayPause,
    Next,
    Previous,
}

pub async fn find_active_player(conn: &Connection) -> Option<String> {
    let dbus = zbus::fdo::DBusProxy::new(conn).await.ok()?;
    let names = dbus.list_names().await.ok()?;
    names
        .into_iter()
        .find(|n| n.starts_with("org.mpris.MediaPlayer2."))
        .map(|n| n.to_string())
}

pub async fn poll_media(conn: &Connection) -> Option<MediaInfo> {
    let service = find_active_player(conn).await?;

    let proxy = MediaPlayer2PlayerProxy::builder(conn)
        .destination(service.as_str())
        .ok()?
        .build()
        .await
        .ok()?;

    let playing = proxy.playback_status().await.ok()? == "Playing";
    let metadata = proxy.metadata().await.ok()?;
    let position_us = proxy.position().await.unwrap_or(0);

    let title = str_from_meta(&metadata, "xesam:title")
        .unwrap_or_else(|| "Inconnu".to_string());
    let artist = str_from_meta(&metadata, "xesam:artist")
        .or_else(|| str_from_meta(&metadata, "xesam:albumArtist"))
        .unwrap_or_else(|| "Inconnu".to_string());
    let duration_us = i64_from_meta(&metadata, "mpris:length").unwrap_or(0);

    Some(MediaInfo {
        title,
        artist,
        progress_secs: (position_us.max(0) / 1_000_000) as u32,
        duration_secs: (duration_us.max(0) / 1_000_000) as u32,
        playing,
        service,
    })
}

pub async fn send_command(conn: &Connection, service: &str, cmd: MediaCommand) {
    let builder = match MediaPlayer2PlayerProxy::builder(conn).destination(service) {
        Ok(b) => b,
        Err(_) => return,
    };
    let proxy = match builder.build().await {
        Ok(p) => p,
        Err(_) => return,
    };
    let _ = match cmd {
        MediaCommand::PlayPause => proxy.play_pause().await,
        MediaCommand::Next => proxy.next().await,
        MediaCommand::Previous => proxy.previous().await,
    };
}

fn str_from_meta(map: &HashMap<String, OwnedValue>, key: &str) -> Option<String> {
    let val = map.get(key)?;
    match val.deref() {
        Value::Str(s) => Some(s.to_string()),
        // xesam:artist is as (array of strings) — Array::inner() gives &[Value<'_>]
        Value::Array(arr) => arr.inner().iter().find_map(|v| {
            if let Value::Str(s) = v { Some(s.to_string()) } else { None }
        }),
        _ => None,
    }
}

fn i64_from_meta(map: &HashMap<String, OwnedValue>, key: &str) -> Option<i64> {
    let val = map.get(key)?;
    match val.deref() {
        Value::I64(n) => Some(*n),
        _ => None,
    }
}
