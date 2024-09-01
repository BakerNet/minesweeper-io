pub mod dark_mode;
pub mod icons;
pub mod info;

pub fn input_class(exta_classes: Option<&str>) -> String {
    let extra_classes = exta_classes.unwrap_or_default();
    format!("flex h-10 w-full border border-blue-950 bg-white text-black px-3 py-2 text-sm disabled:cursor-not-allowed disabled:opacity-50 flex-1 {}", extra_classes)
}

pub fn button_class(extra_classes: Option<&str>, custom_colors: Option<&str>) -> String {
    let colors = custom_colors.unwrap_or("bg-neutral-500 text-neutral-50 hover:bg-neutral-600/90");
    let extra_classes = extra_classes.unwrap_or_default();
    format!("inline-flex items-center justify-center text-md font-medium border border-solid border-black disabled:pointer-events-none disabled:opacity-50 h-10 px-4 py-2 {} {}", colors, extra_classes)
}

pub fn cell_class(content_class: &str, player_class: &str) -> String {
    format!("inline-block text-center border border-solid border-black font-bold align-top h-8 w-8 text-2xl {} {}", content_class, player_class)
}

pub fn number_class(num: usize) -> String {
    String::from(match num {
        1 => "text-blue-600",
        2 => "text-green-600",
        3 => "text-red-600",
        4 => "text-blue-950",
        5 => "text-rose-900",
        6 => "text-teal-600",
        7 => "text-neutral-950",
        8 => "text-neutral-600",
        _ => "",
    })
}

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
