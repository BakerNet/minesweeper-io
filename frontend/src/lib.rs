use std::{cell::RefCell, rc::Rc};

use leptos::{leptos_dom::console_log, *};
use minesweeper::{
    board::BoardPoint,
    cell::PlayerCell,
    game::{Action, Minesweeper, PlayOutcome},
};

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    view! { cx,
        <Game rows=9 cols=9 mines=10 />
    }
}

struct FrontendGame {
    cell_signals: Vec<Vec<WriteSignal<PlayerCell>>>,
    err_signal: WriteSignal<Option<String>>,
    game: Box<Minesweeper>,
}

impl FrontendGame {
    fn click(&mut self, row: usize, col: usize) {
        console_log("Hello from Click");
        let res = self.game.play(0, Action::Click, BoardPoint { row, col });
        match res {
            Err(e) => {
                console_log(&format!("{:?}", e));
                self.err_signal.set(Some(format!("{:?}", e)));
            }
            Ok(outcome) => {
                console_log(&format!("{:?}", outcome));
                self.err_signal.set(None);
                match outcome {
                    PlayOutcome::Success(v) => v.into_iter().for_each(|rc| {
                        self.cell_signals[rc.cell_point.row][rc.cell_point.col]
                            .set(PlayerCell::Revealed(rc));
                    }),
                    PlayOutcome::Victory(v) => v.into_iter().for_each(|rc| {
                        self.cell_signals[rc.cell_point.row][rc.cell_point.col]
                            .set(PlayerCell::Revealed(rc));
                        self.err_signal.set(Some(String::from("VICTORY!!!")));
                    }),
                    PlayOutcome::Failure(rc) => {
                        self.cell_signals[rc.cell_point.row][rc.cell_point.col]
                            .set(PlayerCell::Revealed(rc));
                        self.err_signal.set(Some(String::from("YOU DIED!")));
                    }
                }
            }
        }
    }
}

#[component]
pub fn Game(cx: Scope, rows: usize, cols: usize, mines: usize) -> impl IntoView {
    let game = Minesweeper::init_game(rows, cols, mines, 1).unwrap();
    let curr_board = &game.player_board(0);
    let mut read_signals: Vec<Vec<ReadSignal<PlayerCell>>> = Vec::new();
    let mut write_signals: Vec<Vec<WriteSignal<PlayerCell>>> = Vec::new();
    curr_board.iter().for_each(|v| {
        let mut read_row = Vec::new();
        let mut write_row = Vec::new();
        v.iter().for_each(|c| {
            let (rs, ws) = create_signal(cx, *c);
            read_row.push(rs);
            write_row.push(ws);
        });
        read_signals.push(read_row);
        write_signals.push(write_row);
    });
    let (error, set_error) = create_signal::<Option<String>>(cx, None);
    provide_context(
        cx,
        Rc::new(RefCell::new(FrontendGame {
            cell_signals: write_signals,
            err_signal: set_error,
            game: Box::new(game),
        })),
    );

    view! { cx,
        <div>{
            read_signals
                .into_iter()
                .enumerate()
                .map(move |(row, vec)| view!{cx, <Row row=row cells=vec />})
                .collect_view(cx)
        }</div>
        <div class="error">{error}</div>
    }
}

#[component]
fn Row(cx: Scope, row: usize, cells: Vec<ReadSignal<PlayerCell>>) -> impl IntoView {
    view! { cx,
        <div>{
            cells
                .into_iter()
                .enumerate()
                .map(move |(col, cell)| view! {cx, <Cell row=row col=col cell=cell /> })
                .collect_view(cx)
        }</div>
    }
}

#[component]
fn Cell(cx: Scope, row: usize, col: usize, cell: ReadSignal<PlayerCell>) -> impl IntoView {
    let id = format!("{}_{}", row, col);
    let on_click = move |_| {
        console_log("Hello world");
        let game = use_context::<Rc<RefCell<FrontendGame>>>(cx).unwrap();
        let mut game = (*game).borrow_mut();
        console_log(&format!("{:?}", game.err_signal));
        game.click(row, col);
    };
    view! { cx,
        <span class="cell" id=id on:click=on_click >{move || format!("{:?}", cell()) }</span>
    }
}
