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

#[component]
pub fn Players() -> impl IntoView {
    let game_info = expect_context::<Resource<String, Result<GameInfo, ServerFnError>>>();

    let player_view = move |game_info: GameInfo| match game_info.is_completed {
        true => view! { <InactivePlayers game_info/> },
        false => view! { <ActivePlayers/> },
    };

    view! {
        <Suspense fallback=move || ()>
            {game_info
                .get()
                .map(|game_info| {
                    view! {
                        <ErrorBoundary fallback=|_| {
                            view! { <div class="error">"Unable to load players"</div> }
                        }>{move || { game_info.clone().map(player_view) }}</ErrorBoundary>
                    }
                })}

        </Suspense>
    }
}

#[component]
pub fn ActivePlayers() -> impl IntoView {
    let game = expect_context::<FrontendGame>();
    let (player, players, started) = { (game.player_id, game.players.clone(), game.started) };
    let last_slot = *players.last().unwrap();
    let available_slots = move || last_slot().is_none() && player().is_none();
    let show_start = move || game.game_info.is_owner && !started();

    view! {
        <Show when=available_slots fallback=move || view! { <h4>Scoreboard</h4> }>
            <JoinForm/>
        </Show>
        <table>
            <tr>
                <th>Player</th>
                <th>Username</th>
                <th>Score</th>
            </tr>
            {players
                .iter()
                .enumerate()
                .map(move |(n, &player)| {
                    view! { <ActivePlayer player_num=n player=player/> }
                })
                .collect_view()}
        </table>
        <Show when=show_start fallback=move || ()>
            <StartForm/>
        </Show>
        <A href="..">Hide</A>
    }
}

#[server(GetPlayers, "/api")]
pub async fn get_players(game_id: String) -> Result<Vec<ClientPlayer>, ServerFnError> {
    let game_manager = use_context::<GameManager>()
        .ok_or_else(|| ServerFnError::ServerError("No game manager".to_string()))?;
    let players = game_manager
        .get_players(&game_id)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    Ok(players
        .iter()
        .map(|p| ClientPlayer {
            player_id: p.player as usize,
            username: FrontendUser::display_name_or_anon(&p.display_name),
            dead: p.dead,
            score: p.score as usize,
        })
        .collect())
}

#[component]
pub fn InactivePlayers(game_info: GameInfo) -> impl IntoView {
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
        <table>
            <tr>
                <th>Player</th>
                <th>Username</th>
                <th>Score</th>
            </tr>
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
                                view! { <InactivePlayer player_num=i player=player.clone()/> }
                            })
                            .collect_view(),
                    )
                }}

            </Transition>
        </table>
        <A href="..">Hide</A>
    }
}

#[component]
fn ActivePlayer(player_num: usize, player: ReadSignal<Option<ClientPlayer>>) -> impl IntoView {
    let items = move || {
        if let Some(player) = player() {
            (
                format!("p-{}", player.player_id),
                player.username,
                player.score,
            )
        } else {
            (String::from(""), String::from("--------"), 0)
        }
    };
    view! {
        <tr class=move || items().0>
            <td>{player_num}</td>
            <td>{move || items().1}</td>
            <td>{move || items().2}</td>
        </tr>
    }
}

#[component]
fn InactivePlayer(player_num: usize, player: Option<ClientPlayer>) -> impl IntoView {
    let (class, username, score) = if let Some(player) = &player {
        (
            format!("p-{}", player.player_id),
            player.username.clone(),
            player.score,
        )
    } else {
        (String::from(""), String::from("--------"), 0)
    };
    view! {
        <tr class=class>
            <td>{player_num}</td>
            <td>{username}</td>
            <td>{score}</td>
        </tr>
    }
}

#[derive(Serialize, Debug)]
pub struct PlayForm {
    game_id: String,
    user: String,
}

#[component]
fn JoinForm() -> impl IntoView {
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
                    <form on:submit=move |ev| {
                        ev.prevent_default();
                        join_game();
                    }>
                        <button type="submit">"Join Game"</button>
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
        .ok_or_else(|| ServerFnError::ServerError("Unable to find auth session".to_string()))?;
    let game_manager = use_context::<GameManager>()
        .ok_or_else(|| ServerFnError::ServerError("No game manager".to_string()))?;

    let user = match auth_session.user {
        Some(user) => user,
        None => {
            return Err(ServerFnError::ServerError("Not logged in".to_string()));
        }
    };

    game_manager
        .start_game(&game_id, &user)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
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
                    <ActionForm action=start_game>
                        <input type="hidden" name="game_id" value=game_id/>
                        <button type="submit">"Start Game"</button>
                    </ActionForm>
                }
                    .into_view()
            } else {
                view! { <div>Starting...</div> }.into_view()
            }
        }}
    }
}
