use cfg_if::cfg_if;

use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

use super::{auth::LogOut, FrontendUser};
use crate::{
    app::minesweeper::players::player_class,
    components::{button::Button, icons::Mine, input::input_class},
    no_prefix_serverfnerror, validate_display_name,
};

cfg_if! { if #[cfg(feature="ssr")] {
    use axum_login::AuthUser;
    use super::auth::get_user;
    use crate::backend::{users::AuthSession, game_manager::GameManager};
}}

#[component]
pub fn Profile(
    logout: Action<LogOut, Result<(), ServerFnError>>,
    user: FrontendUser,
    user_updated: WriteSignal<String>,
) -> impl IntoView {
    view! {
        <div class="flex-1 flex flex-col items-center justify-center py-12 px-4 space-y-4">
            <SetDisplayName user user_updated/>
            <div class="w-full max-w-xs h-6">
                <span class="w-full h-full inline-flex items-center justify-center text-lg font-medium text-gray-800 dark:text-gray-200">
                    <hr class="w-full"/>
                </span>
            </div>
            <LogOut logout/>
            <div class="w-full max-w-xs h-6">
                <span class="w-full h-full inline-flex items-center justify-center text-lg font-medium text-gray-800 dark:text-gray-200">
                    <hr class="w-full"/>
                </span>
            </div>
            <GameHistory/>
        </div>
    }
}

#[server(SetDisplayName, "/api")]
async fn set_display_name(display_name: String) -> Result<String, ServerFnError> {
    if !validate_display_name(&display_name) {
        return Err(ServerFnError::new("Display name not valid".to_string()));
    }
    let user = get_user()
        .await?
        .ok_or_else(|| ServerFnError::new("Unable to find user".to_string()))?;
    if let Some(name) = &user.display_name {
        if name == &display_name {
            return Ok(display_name);
        }
    }
    let auth_session = use_context::<AuthSession>().unwrap();
    auth_session
        .backend
        .update_user_display_name(user.id(), &display_name)
        .await
        .map(|_| display_name)
        .map_err(|_| ServerFnError::new("Unable to update display name".to_string()))
}

#[component]
fn SetDisplayName(user: FrontendUser, user_updated: WriteSignal<String>) -> impl IntoView {
    let set_display_name = create_server_action::<SetDisplayName>();
    let (name_err, set_name_err) = create_signal::<Option<String>>(None);

    let on_submit = move |ev| {
        let data = SetDisplayName::from_event(&ev);
        if data.is_err() || !validate_display_name(&data.unwrap().display_name) {
            ev.prevent_default();
            set_name_err(Some("Display name not valid".to_string()));
        }
    };

    create_effect(move |_| match set_display_name.value().get() {
        Some(Ok(name)) => {
            user_updated(name);
            set_name_err(None);
        }
        Some(Err(e)) => set_name_err(Some(
            no_prefix_serverfnerror(e) + ". This display name may already be taken",
        )),
        _ => {}
    });

    let curr_name = FrontendUser::display_name_or_anon(&user.display_name, true);

    view! {
        <div class="flex flex-col space-y-2 w-full max-w-xs">
            <span class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 text-neutral-950 dark:text-neutral-50">
                {curr_name.clone()}
            </span>
            {move || {
                name_err
                    .get()
                    .map(|s| {
                        view! {
                            <span class="text-sm font-medium leading-none text-red-500">{s}</span>
                        }
                    })
            }}

            <ActionForm
                action=set_display_name
                on:submit=move |e| on_submit(e.into())
                class="flex space-x-2"
            >
                <input
                    class=input_class("")
                    type="text"
                    id="set_display_name_display_name"
                    name="display_name"
                    placeholder=curr_name
                />
                <Button btn_type="submit">"Set display name"</Button>
            </ActionForm>
        </div>
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerGame {
    game_id: String,
    player_id: u8,
    dead: bool,
    score: i64,
}

#[server(GetPlayerGames, "/api")]
async fn get_player_games() -> Result<Vec<PlayerGame>, ServerFnError> {
    let auth_session = use_context::<AuthSession>()
        .ok_or_else(|| ServerFnError::new("Unable to find auth session".to_string()))?;
    let user = auth_session.user.ok_or(ServerFnError::new(
        "Cannot find player games when not logged in".to_string(),
    ))?;
    let game_manager = use_context::<GameManager>()
        .ok_or_else(|| ServerFnError::new("No game manager".to_string()))?;

    let games = game_manager
        .get_player_games_for_user(&user)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(games
        .into_iter()
        .map(|pu| PlayerGame {
            game_id: pu.game_id,
            player_id: pu.player,
            dead: pu.dead,
            score: pu.score,
        })
        .collect())
}

#[component]
fn GameHistory() -> impl IntoView {
    let player_games = create_resource(|| (), move |_| async { get_player_games().await });

    let game_view = move |game: PlayerGame| {
        let player_class = player_class(game.player_id as usize) + " text-black";
        view! {
            <tr class=player_class>
                <td class="border-b border-slate-100 dark:border-slate-700 p-1">
                    <A
                        class="text-sky-800 hover:text-sky-500 font-medium"
                        href=format!("/game/{}/players", game.game_id)
                    >
                        {game.game_id}
                    </A>
                </td>
                <td class="border-b border-slate-100 dark:border-slate-700 p-1">
                    {if game.dead {
                        view! {
                            <span class="inline-block align-text-top bg-red-600 h-4 w-4">
                                <Mine/>
                            </span>
                        }
                            .into_view()
                    } else {
                        ().into_view()
                    }}

                </td>
                <td class="border-b border-slate-100 dark:border-slate-700 p-1">{game.score}</td>
            </tr>
        }
    };
    view! {
        <h4 class="text-2xl my-4 text-gray-900 dark:text-gray-200">Game History</h4>
        <table class="border border-solid border-slate-400 border-collapse table-auto w-full max-w-xs text-sm text-center">
            <thead>
                <tr>
                    <th class="border-b dark:border-slate-600 font-medium p-4 text-slate-400 dark:text-slate-200 ">
                        Game
                    </th>
                    <th class="border-b dark:border-slate-600 font-medium p-4 text-slate-400 dark:text-slate-200 ">
                        Status
                    </th>
                    <th class="border-b dark:border-slate-600 font-medium p-4 text-slate-400 dark:text-slate-200 ">
                        Score
                    </th>
                </tr>
            </thead>
            <tbody>
                <Suspense fallback=move || ()>

                    {move || {
                        player_games
                            .get()
                            .map(|games| {
                                games
                                    .map(move |games| {
                                        games.into_iter().map(game_view).collect_view()
                                    })
                            })
                    }}

                </Suspense>
            </tbody>
        </table>
    }
}
