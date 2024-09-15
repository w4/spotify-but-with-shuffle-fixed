use futures::StreamExt;
use rand::prelude::SliceRandom;
use rspotify::{
    clients::{BaseClient, OAuthClient},
    model::{AlbumId, FullTrack, PlayableId, PlayableItem, PlaylistId, Type},
    scopes, AuthCodePkceSpotify, Config, Credentials, OAuth,
};
use std::time::Duration;
use tracing::{debug, info, warn};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::fmt().pretty().init();

    let mut spotify = AuthCodePkceSpotify::with_config(
        Credentials::new_pkce("b6146c081df54ae79e42258a8619f570"),
        OAuth {
            scopes: scopes!("user-read-playback-state user-modify-playback-state playlist-read-private user-library-read"),
            redirect_uri: "http://127.0.0.1/".to_string(),
            ..OAuth::default()
        },
        Config {
            token_cached: true,
            ..Config::default()
        },
    );

    let url = spotify.get_authorize_url(None).unwrap();
    spotify.prompt_for_token(&url).await.unwrap();

    let mut tracklist = vec![];
    let mut current_context = None;
    let mut now_playing = None;
    let mut rng = rand::thread_rng();

    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;

        let playback = match spotify.current_playback(None, None::<Vec<_>>).await {
            Ok(Some(v)) => v,
            Ok(None) => {
                debug!("Nothing currently playing");
                continue;
            }
            Err(e) => {
                warn!("Failed to send Spotify request: {e}");
                continue;
            }
        };

        if !playback.is_playing || !playback.shuffle_state {
            debug!("User is not currently shuffling");
            continue;
        }

        if current_context.as_deref() != playback.context.as_ref().map(|v| v.uri.as_str()) {
            // user has changed the playlist/album/etc they're playing from so
            // we need to refresh our tracklist.
            let kind = playback.context.as_ref().map(|v| v._type);
            current_context = playback.context.map(|v| v.uri);
            tracklist.clear();

            info!("User now playing from {current_context:?}, refreshing tracklist");

            let (Some(kind), Some(uri)) = (kind, current_context.as_deref()) else {
                continue;
            };

            fetch_track_list(kind, uri, &mut tracklist, &spotify).await;
            tracklist.shuffle(&mut rng);

            info!(count = tracklist.len(), "Fetched tracks & shuffled");
        }

        if now_playing != playback.item.as_ref().and_then(|v| v.id()) {
            now_playing = playback.item.and_then(into_playable_id);

            let Some(to_queue) = tracklist.pop() else {
                warn!("Ran out of tracks to queue");
                continue;
            };

            info!("Track changed to {now_playing:?}, queueing track: {to_queue:?}");

            if let Err(e) = spotify.add_item_to_queue(to_queue, None).await {
                warn!("Failed to add item to queue: {e}");
            }
        }
    }
}

async fn fetch_track_list(
    kind: Type,
    uri: &str,
    tracklist: &mut Vec<PlayableId<'static>>,
    spotify: &AuthCodePkceSpotify,
) {
    match kind {
        Type::Artist
        | Type::Track
        | Type::User
        | Type::Show
        | Type::Episode
        | Type::Collectionyourepisodes => {
            warn!("Unsupported context type: {kind:?}");
        }
        Type::Album => {
            let mut stream = spotify.album_track(AlbumId::from_uri(uri).unwrap(), None);

            while let Some(track) = stream.next().await {
                let track = match track {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("Failed to fetch tracks from album: {e}");
                        break;
                    }
                };

                if let Some(id) = track.id {
                    tracklist.push(PlayableId::Track(id));
                }
            }
        }
        Type::Playlist => {
            let mut stream = spotify.playlist_items(PlaylistId::from_uri(uri).unwrap(), None, None);

            while let Some(item) = stream.next().await {
                let item = match item {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("Failed to fetch tracks from playlist: {e}");
                        break;
                    }
                };

                if let Some(id) = item.track.and_then(into_playable_id) {
                    tracklist.push(id);
                }
            }
        }
        Type::Collection => {
            let mut stream = spotify.current_user_saved_tracks(None);

            while let Some(track) = stream.next().await {
                let track = match track {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("Failed to fetch tracks from album: {e}");
                        break;
                    }
                };

                if let Some(id) = track.track.id {
                    tracklist.push(PlayableId::Track(id));
                }
            }
        }
    }
}

fn into_playable_id(item: PlayableItem) -> Option<PlayableId<'static>> {
    match item {
        PlayableItem::Track(FullTrack { id: Some(id), .. }) => Some(PlayableId::Track(id)),
        PlayableItem::Episode(episode) => Some(PlayableId::Episode(episode.id)),
        _ => None,
    }
}
