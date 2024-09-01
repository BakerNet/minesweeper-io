pub mod dark_mode;
pub mod icons;
pub mod info;

#[macro_export]
macro_rules! input_class {
    () => {"flex h-10 w-full border border-blue-950 bg-white text-black px-3 py-2 text-sm disabled:cursor-not-allowed disabled:opacity-50 flex-1"};
    ($extra:literal) => {concat!("flex h-10 w-full border border-blue-950 bg-white text-black px-3 py-2 text-sm disabled:cursor-not-allowed disabled:opacity-50 flex-1 ", $extra)};
}

#[macro_export]
macro_rules! button_class {
    () => {
        "inline-flex items-center justify-center text-md font-medium border border-solid border-black disabled:pointer-events-none disabled:opacity-50 h-10 px-4 py-2 bg-neutral-500 text-neutral-50 hover:bg-neutral-600/90"
    };
    ($extra:literal) => {
        concat!("inline-flex items-center justify-center text-md font-medium border border-solid border-black disabled:pointer-events-none disabled:opacity-50 h-10 px-4 py-2 bg-neutral-500 text-neutral-50 hover:bg-neutral-600/90 ", $extra)
    };
    ($extra:literal, $colors:literal) => {
        concat!("inline-flex items-center justify-center text-md font-medium border border-solid border-black disabled:pointer-events-none disabled:opacity-50 h-10 px-4 py-2 ", $extra, " ", $colors)
    };
}

#[macro_export]
macro_rules! cell_class {
    ($extra:literal) => {
        concat!("inline-block text-center border border-solid border-black font-bold align-top h-8 w-8 text-2xl ", $extra)
    };
    ($extra:literal, $colors:literal) => {
        concat!("inline-block text-center border border-solid border-black font-bold align-top h-8 w-8 text-2xl ", $extra, " ", $colors)
    };
    ($extra:expr) => {
        format!("inline-block text-center border border-solid border-black font-bold align-top h-8 w-8 text-2xl {}", $extra)
    };
    ($extra:expr, $colors:expr) => {
        format!("inline-block text-center border border-solid border-black font-bold align-top h-8 w-8 text-2xl {} {}", $extra, $colors)
    };
}

#[macro_export]
macro_rules! number_class {
    (1) => {
        "text-blue-600"
    };
    (2) => {
        "text-green-600"
    };
    (3) => {
        "text-red-600"
    };
    (4) => {
        "text-blue-950"
    };
    (5) => {
        "text-rose-900"
    };
    (6) => {
        "text-teal-600"
    };
    (7) => {
        "text-neutral-950"
    };
    (8) => {
        "text-neutral-600"
    };
    ($s:expr) => {
        match $s {
            1 => number_class!(1),
            2 => number_class!(2),
            3 => number_class!(3),
            4 => number_class!(4),
            5 => number_class!(5),
            6 => number_class!(6),
            7 => number_class!(7),
            8 => number_class!(8),
            _ => "",
        }
    };
}

#[macro_export]
macro_rules! player_class {
    ( 0 ) => {
        "bg-cyan-200"
    };
    ( 1 ) => {
        "bg-indigo-200"
    };
    ( 2 ) => {
        "bg-fuchsia-200"
    };
    ( 3 ) => {
        "bg-orange-200"
    };
    ( 4 ) => {
        "bg-lime-200"
    };
    ( 5 ) => {
        "bg-teal-200"
    };
    ( 6 ) => {
        "bg-blue-200"
    };
    ( 7 ) => {
        "bg-purple-200"
    };
    ( 8 ) => {
        "bg-rose-200"
    };
    ( 9 ) => {
        "bg-yellow-200"
    };
    ( 10 ) => {
        "bg-emerald-200"
    };
    ( 11 ) => {
        "bg-sky-200"
    };
    ($s:expr) => {
        match $s {
            0 => player_class!(0),
            1 => player_class!(1),
            2 => player_class!(2),
            3 => player_class!(3),
            4 => player_class!(4),
            5 => player_class!(5),
            6 => player_class!(6),
            7 => player_class!(7),
            8 => player_class!(8),
            9 => player_class!(9),
            10 => player_class!(10),
            11 => player_class!(11),
            _ => "",
        }
    };
}
