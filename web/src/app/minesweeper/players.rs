use super::{client::FrontendGame, GameInfo};

use anyhow::Result;
use leptos::*;
use leptos_router::*;
use minesweeper_lib::client::ClientPlayer;
use serde::Serialize;

#[cfg(feature = "ssr")]
use crate::app::FrontendUser;
#[cfg(feature = "ssr")]
use crate::backend::{game_manager::GameManager, users::AuthSession};
use crate::components::button::Button;
use crate::components::icons::Mine;

pub fn player_class(player: usize) -> String {
    String::from(match player {
        0 => "bg-cyan-200",
        1 => "bg-indigo-200",
        2 => "bg-fuchsia-200",
        3 => "bg-orange-200",
        4 => "bg-lime-200",
        5 => "bg-teal-200",
        6 => "bg-blue-200",
        7 => "bg-purple-200",
        8 => "bg-rose-200",
        9 => "bg-yellow-200",
        10 => "bg-emerald-200",
        11 => "bg-sky-200",
        _ => "",
    })
}

#[component]
pub fn ShowPlayers() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center my-8">
            <A
                href="players"
                class="text-lg text-gray-700 dark:text-gray-400 hover:text-sky-800 dark:hover:text-sky-500"
            >
                "Show Scoreboard"
            </A>
        </div>
    }
}

#[component]
pub fn Players() -> impl IntoView {
    let game_info = expect_context::<Resource<String, Result<GameInfo, ServerFnError>>>();

    let player_view = move |game_info: GameInfo| match game_info.is_completed {
        true => view! { <InactivePlayers game_info/> },
        false => view! { <ActivePlayers/> },
    };

    view! {
        <div class="flex flex-col items-center my-8 space-y-4">
            <Suspense fallback=move || ()>
                {game_info
                    .get()
                    .map(|game_info| {
                        view! {
                            <ErrorBoundary fallback=|_| {
                                view! { <div class="text-red-600">"Unable to load players"</div> }
                            }>{move || { game_info.clone().map(player_view) }}</ErrorBoundary>
                        }
                    })}

            </Suspense>
        </div>
    }
}

#[component]
fn Scoreboard<F, IV>(children: Children, buttons: F) -> impl IntoView
where
    F: Fn() -> IV,
    IV: IntoView,
{
    view! {
        <h4 class="text-2xl my-4 text-gray-900 dark:text-gray-200">Scoreboard</h4>
        <table class="border border-solid border-slate-400 border-collapse table-auto w-full max-w-xs text-sm text-center">
            <thead>
                <tr>
                    <th class="border-b dark:border-slate-600 font-medium p-4 text-slate-400 dark:text-slate-200 ">
                        Player
                    </th>
                    <th class="border-b dark:border-slate-600 font-medium p-4 text-slate-400 dark:text-slate-200 ">
                        Username
                    </th>
                    <th class="border-b dark:border-slate-600 font-medium p-4 text-slate-400 dark:text-slate-200 ">
                        Score
                    </th>
                </tr>
            </thead>
            <tbody>{children()}</tbody>
        </table>
        {buttons()}
        <A
            href=".."
            class="text-gray-700 dark:text-gray-400 hover:text-sky-800 dark:hover:text-sky-500"
        >
            Hide
        </A>
    }
}

#[component]
fn ActivePlayers() -> impl IntoView {
    let game = expect_context::<FrontendGame>();

    let (player, players, loaded, started) = {
        (
            game.player_id,
            game.players.clone(),
            game.players_loaded,
            game.started,
        )
    };
    let last_slot = *players.last().unwrap();
    let available_slots = move || loaded() && last_slot().is_none() && player().is_none();
    let show_start = move || {
        loaded()
            && (game.game_info.is_owner || (!game.game_info.has_owner && player().is_some()))
            && !started()
    };

    let buttons = move || {
        view! {
            <Show when=available_slots fallback=move || ()>
                <PlayForm/>
            </Show>
            <Show when=show_start fallback=move || ()>
                <StartForm/>
            </Show>
        }
    };

    view! {
        <Scoreboard buttons>
            {players
                .iter()
                .enumerate()
                .map(move |(n, &player)| {
                    view! { <ActivePlayer player_num=n player=player/> }
                })
                .collect_view()}
        </Scoreboard>
    }
}

#[server(GetPlayers, "/api")]
pub async fn get_players(game_id: String) -> Result<Vec<ClientPlayer>, ServerFnError> {
    let game_manager = use_context::<GameManager>()
        .ok_or_else(|| ServerFnError::new("No game manager".to_string()))?;
    let players = game_manager
        .get_players(&game_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(players
        .iter()
        .map(|p| ClientPlayer {
            player_id: p.player as usize,
            username: FrontendUser::display_name_or_anon(&p.display_name, p.user.is_some()),
            dead: p.dead,
            score: p.score as usize,
        })
        .collect())
}

#[component]
fn InactivePlayers(game_info: GameInfo) -> impl IntoView {
    let (game_info, _) = create_signal((game_info.max_players, game_info.game_id));
    let players = create_resource(game_info, |game_info| async move {
        let players = get_players(game_info.1.clone()).await.ok();
        players.map(|pv| {
            let mut players = vec![None; game_info.0 as usize];
            pv.iter()
                .for_each(|p| players[p.player_id] = Some(p.clone()));
            players
        })
    });
    view! {
        <Scoreboard buttons=move || ()>
            <Transition fallback=move || {
                view! {}
            }>
                {move || {
                    let players = players.get().flatten()?;
                    Some(
                        players
                            .iter()
                            .enumerate()
                            .map(|(i, player)| {
                                view! { <PlayerRow player_num=i player=player.clone()/> }
                            })
                            .collect_view(),
                    )
                }}

            </Transition>
        </Scoreboard>
    }
}

#[component]
fn ActivePlayer(player_num: usize, player: ReadSignal<Option<ClientPlayer>>) -> impl IntoView {
    view! {
        {move || {
            view! { <PlayerRow player_num=player_num player=player()/> }
        }}
    }
}

#[component]
fn PlayerRow(player_num: usize, player: Option<ClientPlayer>) -> impl IntoView {
    let (mut player_class, username, is_dead, score) = if let Some(player) = &player {
        (
            player_class(player.player_id),
            player.username.clone(),
            player.dead,
            player.score,
        )
    } else {
        (String::from(""), String::from("--------"), false, 0)
    };
    if player_class != "" {
        player_class += " text-black";
    } else {
        player_class = "text-slate-600 dark:text-slate-400".to_string();
    }

    view! {
        <tr class=player_class>
            <td class="border-b border-slate-100 dark:border-slate-700 p-1">{player_num}</td>
            <td class="border-b border-slate-100 dark:border-slate-700 p-1">
                {username}
                {if is_dead {
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
            <td class="border-b border-slate-100 dark:border-slate-700 p-1">{score}</td>
        </tr>
    }
}

#[derive(Serialize, Debug)]
pub struct PlayForm {
    game_id: String,
    user: String,
}

#[component]
fn PlayForm() -> impl IntoView {
    let game = expect_context::<FrontendGame>();
    let (game, _) = create_signal(game);
    let (show, set_show) = create_signal(true);

    let join_game = move || {
        game().send("Play");
        set_show(false);
    };

    view! {
        {move || {
            if show() {
                view! {
                    <form
                        on:submit=move |ev| {
                            ev.prevent_default();
                            join_game();
                        }

                        class="w-full max-w-xs h-8"
                    >
                        <Button btn_type="submit" class="w-full w-max-xs h-8">
                            "Play Game"
                        </Button>
                    </form>
                }
                    .into_view()
            } else {
                view! { <div>Joining...</div> }.into_view()
            }
        }}
    }
}

#[server(StartGame, "/api")]
async fn start_game(game_id: String) -> Result<(), ServerFnError> {
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
fn StartForm() -> impl IntoView {
    let game = expect_context::<FrontendGame>();
    let start_game = create_server_action::<StartGame>();
    let (game_id, _) = create_signal(game.game_info.game_id);

    let show = move || !start_game.pending().get();

    view! {
        {move || {
            if show() {
                view! {
                    <ActionForm action=start_game class="w-full max-w-xs h-8">
                        <input type="hidden" name="game_id" value=game_id/>
                        <Button
                            btn_type="submit"
                            class="w-full w-max-xs h-8"
                            colors="bg-green-700 hover:bg-green-800/90 text-white"
                        >
                            "Start Game"
                        </Button>
                    </ActionForm>
                }
                    .into_view()
            } else {
                view! { <div>Starting...</div> }.into_view()
            }
        }}
    }
}
