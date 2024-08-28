use leptos::*;
use leptos_router::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::{
    auth::{FrontendUser, LogOut},
    minesweeper::GameMode,
};
use crate::components::{
    button_class,
    icons::{player_icon_holder, IconTooltip, Mine, Star, Trophy},
    input_class, player_class,
};

#[cfg(feature = "ssr")]
use super::{auth::get_user, minesweeper::GameSettings};
#[cfg(feature = "ssr")]
use crate::backend::{AuthSession, GameManager};
#[cfg(feature = "ssr")]
use axum_login::AuthUser;

fn no_prefix_serverfnerror(s: ServerFnError) -> String {
    s.to_string()
        .split(": ")
        .last()
        .expect("ServerFnError String expected to have prefix")
        .to_string()
}

fn validate_display_name(name: &str) -> bool {
    let re = Regex::new(r"^[\w]+$").unwrap();
    re.is_match(name) && name.len() >= 3 && name.len() <= 16
}

#[component]
pub fn Profile<S>(
    user: Resource<S, Option<FrontendUser>>,
    logout: Action<LogOut, Result<(), ServerFnError>>,
    user_updated: WriteSignal<String>,
) -> impl IntoView
where
    S: PartialEq + Clone + 'static,
{
    let user_profile = move |user: Option<FrontendUser>| match user {
        Some(user) => View::from(view! {
            <>
                <div class="flex-1 flex flex-col items-center justify-center py-12 px-4 space-y-4">
                    <SetDisplayName user user_updated />
                    <div class="w-full max-w-xs h-6">
                        <span class="w-full h-full inline-flex items-center justify-center text-lg font-medium text-gray-800 dark:text-gray-200">
                            <hr class="w-full" />
                        </span>
                    </div>
                    <LogOut logout />
                    <div class="w-full max-w-xs h-6">
                        <span class="w-full h-full inline-flex items-center justify-center text-lg font-medium text-gray-800 dark:text-gray-200">
                            <hr class="w-full" />
                        </span>
                    </div>
                    <GameHistory />
                </div>
            </>
        }),
        _ => view! { <Redirect path="/auth/login" /> },
    };

    view! { <Suspense fallback=move || ()>{move || { user.get().map(user_profile) }}</Suspense> }
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

    let curr_name = FrontendUser::display_name_or_anon(user.display_name.as_ref(), true);

    view! {
        <div class="flex flex-col space-y-2 w-full max-w-xs">
            <span class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 text-neutral-950 dark:text-neutral-50">
                {&curr_name}
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
                    class=input_class(None)
                    type="text"
                    id="set_display_name_display_name"
                    name="display_name"
                    placeholder=curr_name
                />
                <button
                    btn_type="submit"
                    class=button_class(None, None)
                    disabled=set_display_name.pending()
                >
                    "Set display name"
                </button>
            </ActionForm>
        </div>
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerGame {
    game_id: String,
    player: u8,
    dead: bool,
    victory_click: bool,
    top_score: bool,
    score: i64,
    start_time: Option<String>,
    game_time: Option<usize>,
    game_mode: GameMode,
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
            player: pu.player,
            dead: pu.dead,
            victory_click: pu.victory_click,
            top_score: pu.top_score,
            score: pu.score,
            start_time: pu
                .start_time
                .map(|dt| dt.date_naive().format("%Y-%m-%d").to_string()),
            game_time: match (pu.start_time, pu.end_time) {
                (Some(st), Some(et)) => {
                    Some(999.min(et.signed_duration_since(st).num_seconds() as usize))
                }
                _ => None,
            },
            game_mode: GameMode::from(GameSettings::new(
                pu.rows,
                pu.cols,
                pu.num_mines,
                pu.max_players.into(),
            )),
        })
        .collect())
}

#[component]
fn GameHistory() -> impl IntoView {
    let player_games = create_resource(|| (), move |_| async { get_player_games().await });
    let td_class = "border border-slate-100 dark:border-slate-700 p-1";

    let loading_row = move |num: usize| {
        let player_class = player_class(0) + " text-black";
        view! {
            <tr class=player_class>
                <td class=td_class>"Game "{num}</td>
                <td class=td_class></td>
                <td class=td_class></td>
                <td class=td_class>"Loading..."</td>
                <td class=td_class></td>
                <td class=td_class></td>
            </tr>
        }
    };
    let game_view = move |game: PlayerGame| {
        let player_class = player_class(game.player as usize) + " text-black";
        view! {
            <tr class=player_class>
                <td class=td_class>
                    <A
                        class="text-sky-800 hover:text-sky-500 font-medium"
                        href=format!("/game/{}", game.game_id)
                    >
                        {game.game_id}
                    </A>
                </td>
                <td class=td_class>{game.start_time}</td>
                <td class=td_class>{game.game_mode.long_name()}</td>
                <td class=td_class>{game.game_time}</td>
                <td class=td_class>
                    {if game.dead {
                        view! {
                            <span class=player_icon_holder("bg-red-600", true)>
                                <Mine />
                                <IconTooltip>"Dead"</IconTooltip>
                            </span>
                        }
                            .into_view()
                    } else {
                        ().into_view()
                    }}
                    {if game.top_score {
                        view! {
                            <span class=player_icon_holder("bg-green-800", true)>
                                <Trophy />
                                <IconTooltip>"Top Score"</IconTooltip>
                            </span>
                        }
                            .into_view()
                    } else {
                        ().into_view()
                    }}
                    {if game.victory_click {
                        view! {
                            <span class=player_icon_holder("bg-black", true)>
                                <Star />
                                <IconTooltip>"Victory Click"</IconTooltip>
                            </span>
                        }
                            .into_view()
                    } else {
                        ().into_view()
                    }}

                </td>
                <td class=td_class>{game.score}</td>
            </tr>
        }
    };
    view! {
        <h4 class="text-2xl my-4 text-gray-900 dark:text-gray-200">"Game History"</h4>
        <div class="max-w-full overflow-x-auto">
        <table class="border border-solid border-slate-400 border-collapse table-auto text-sm text-center">
            <thead>
                <tr>
                    <th class="border dark:border-slate-600 font-medium p-4 text-slate-400 dark:text-slate-200 ">
                        "Game"
                    </th>
                    <th class="border dark:border-slate-600 font-medium p-4 text-slate-400 dark:text-slate-200 ">
                        "Date"
                    </th>
                    <th class="border dark:border-slate-600 font-medium p-4 text-slate-400 dark:text-slate-200 ">
                        "Game Mode"
                    </th>
                    <th class="border dark:border-slate-600 font-medium p-4 text-slate-400 dark:text-slate-200 ">
                        "Duration"
                    </th>
                    <th class="border dark:border-slate-600 font-medium p-4 text-slate-400 dark:text-slate-200 ">
                        "Status"
                    </th>
                    <th class="border dark:border-slate-600 font-medium p-4 text-slate-400 dark:text-slate-200 ">
                        "Score"
                    </th>
                </tr>
            </thead>
            <tbody>
                <Suspense fallback=move || {
                    (0..5).map(loading_row).collect_view()
                }>

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
        </div>
    }
}
