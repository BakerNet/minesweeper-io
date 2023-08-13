mod game;

use game::Game;

use leptos::*;
use leptos_router::*;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    view! { cx,
        <div id="root">
            <Router>
                <h1>Minesweeper</h1>
                <A href="">Home</A>
            <main>
            <Routes>
                // TODO - new game & join game suspense
                <Route path="" view=|cx| view!{cx, <A href="jFSUQSLk">Start game</A>} />
                <Route path="/:id" view=|cx| view!{ cx,
                    <Game rows=16 cols=30 />
                } />
            </Routes>
            </main>
            </Router>
        </div>
    }
}

#[component]
fn StartGame(cx: Scope) -> impl IntoView {
    // let new_game = create_action(cx, move |_| todo!());
    view! {cx,
        todo!()
    }
}
