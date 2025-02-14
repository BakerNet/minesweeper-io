use anyhow::Result;
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use game_manager::GameManager;
use game_ui::button_class;
#[cfg(feature = "ssr")]
use web_auth::AuthSession;

use super::client::FrontendGame;

#[component]
pub fn PlayerButtons(game: StoredValue<FrontendGame>) -> impl IntoView {
    let start_game = ServerAction::<StartGame>::new();

    let FrontendGame {
        game_id,
        is_owner,
        has_owner,
        player_id,
        players,
        players_loaded,
        started,
        join_trigger,
        ..
    } = game.get_value();
    let num_players = players.len();
    let last_slot = *players.last().unwrap();
    let show_play = move || {
        players_loaded() && last_slot().is_none() && player_id().is_none() && num_players > 1
    };
    let show_start = move || {
        players_loaded()
            && (is_owner || (!has_owner && player_id().is_some()))
            && !started()
            && num_players > 1
    };

    if num_players == 1 {
        log::debug!("num players 1");
        Effect::watch(
            players_loaded,
            move |loaded, _, prev| {
                if *loaded && prev.unwrap_or(true) {
                    log::debug!("join_trigger");
                    join_trigger.notify();
                }
                !*loaded
            },
            false,
        );
    }

    view! {
        <Show when=show_play fallback=move || ()>
            <PlayForm join_trigger />
        </Show>
        <Show when=show_start>
            <StartForm start_game game_id=game_id.to_string() />
        </Show>
    }
}

#[component]
fn PlayForm(join_trigger: Trigger) -> impl IntoView {
    let (show, set_show) = signal(true);

    let join_game = move || {
        join_trigger.notify();
        set_show(false);
    };

    view! {
        <Show
            when=show
            fallback=move || {
                view! { <div>"Joining..."</div> }
            }
        >
            <form
                on:submit=move |ev| {
                    ev.prevent_default();
                    join_game();
                }

                class="w-full max-w-xs h-8"
            >
                <button type="submit" class=button_class!("w-full max-w-xs h-8")>
                    "Play Game"
                </button>
            </form>
        </Show>
    }
}

#[server]
pub async fn start_game(game_id: String) -> Result<(), ServerFnError> {
    let auth_session = use_context::<AuthSession>()
        .ok_or_else(|| ServerFnError::new("Unable to find auth session".to_string()))?;
    let game_manager = use_context::<GameManager>()
        .ok_or_else(|| ServerFnError::new("No game manager".to_string()))?;

    game_manager
        .start_game(&game_id, &auth_session.user)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

#[component]
fn StartForm(start_game: ServerAction<StartGame>, game_id: String) -> impl IntoView {
    view! {
        <ActionForm action=start_game attr:class="w-full max-w-xs h-8">
            <input type="hidden" name="game_id" value=game_id />
            <button
                type="submit"
                class=button_class!(
                    "w-full max-w-xs h-8",
                    "bg-green-700 hover:bg-green-800/90 text-white"
                )

                disabled=start_game.pending()
            >
                "Start Game"
            </button>
        </ActionForm>
    }
}
