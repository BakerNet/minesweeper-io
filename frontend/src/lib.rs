mod game;

use game::Game;

use anyhow::Result;
use leptos::*;
use leptos_router::*;
use reqwasm::http::Request;

use crate::game::players::Players;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    view! { cx,
        <div id="root">
            <Router>
                <A href=""><h1 class="logo">Minesweeper</h1></A>
            <main>
            <Routes>
                // TODO - new game & join game suspense
                <Route path="" view=|cx| view!{cx, <StartGame />} />
                <Route path="/:id" view=|cx| view!{ cx,
                    <Game rows=16 cols=30 />
                } >
                    <Route path="players" view=|cx| view! { cx, <Players /> } />
                    <Route path="" view=|cx| view! {cx, <A href="players">"Join Game / Scoreboard"</A>} />
                </Route>
            </Routes>
            </main>
            </Router>
        </div>
    }
}

#[component]
fn StartGame(cx: Scope) -> impl IntoView {
    let new_game: Action<(), Result<()>> = create_action(cx, move |_: &()| async move {
        let navigate = use_navigate(cx);
        let id = Request::post("http://127.0.0.1:3000/api/new")
            .send()
            .await?
            .text()
            .await?;
        request_animation_frame(move || {
            let _ = navigate(&format!("/{}", id), Default::default());
        });
        Result::Ok(())
    });
    view! {cx,
        <form
            on:submit=move |ev| {
                ev.prevent_default(); // don't reload the page...
                new_game.dispatch(());
            }
         >
            <button type="submit">"New Game"</button>
        </form>
    }
}
