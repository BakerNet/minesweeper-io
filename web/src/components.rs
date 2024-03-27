pub mod dark_mode;
pub mod icons;

pub fn input_class(exta_classes: Option<&str>) -> String {
    let extra_classes = exta_classes.unwrap_or_default();
    format!("flex h-10 w-full border border-blue-950 bg-white px-3 py-2 text-sm disabled:cursor-not-allowed disabled:opacity-50 flex-1 {}", extra_classes)
}

pub fn button_class(extra_classes: Option<&str>, custom_colors: Option<&str>) -> String {
    let colors = custom_colors.unwrap_or("bg-neutral-500 text-neutral-50 hover:bg-neutral-600/90");
    let extra_classes = extra_classes.unwrap_or_default();
    format!("inline-flex items-center justify-center text-md font-medium border border-solid border-black disabled:pointer-events-none disabled:opacity-50 h-10 px-4 py-2 {} {}", colors, extra_classes)
}
