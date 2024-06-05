use leptos::*;

pub fn player_icon_holder(bg_color: &str, has_tooltip: bool) -> String {
    let mut class = format!("inline-block align-text-top {bg_color} h-5 w-5 p-0.5 mx-1 rounded");
    if has_tooltip {
        class += " has-tooltip relative"
    }
    class
}

#[component]
pub fn IconTooltip(children: Children) -> impl IntoView {
    view! {
        <span class="tooltip font-bold rounded whitespace-nowrap bg-white text-black px-1 top-0 left-0 -mt-5">
            {children()}
        </span>
    }
}

#[component]
pub fn Flag() -> impl IntoView {
    view! {
        <svg
            viewBox="0 0 11.90625 11.90625"
            version="1.1"
            id="svg742"
            xmlns="http://www.w3.org/2000/svg"
            class="object-cover h-full w-full"
        >
            <g id="layer1">
                <rect
                    style="fill:#000000;stroke-width:0.344195"
                    id="rect746"
                    width="0.93562794"
                    height="8.8635416"
                    x="1.9182292"
                    y="1.7197917"
                    ry="0"
                ></rect>
                <rect
                    style="fill:#ff0000;stroke-width:0.219945"
                    id="rect748"
                    width="4.0933728"
                    height="4.727222"
                    x="2.3860433"
                    y="1.7197917"
                ></rect>
                <rect
                    style="fill:#ff0000;stroke-width:0.214375"
                    id="rect748-5"
                    width="4.0933728"
                    height="4.4908609"
                    x="5.8946481"
                    y="2.901597"
                ></rect>
            </g>
        </svg>
    }
}

#[component]
pub fn Mine() -> impl IntoView {
    view! {
        <svg
            viewBox="0 0 11.90625 11.90625"
            version="1.1"
            id="svg742"
            xmlns="http://www.w3.org/2000/svg"
            class="object-cover h-full w-full"
        >
            <g id="layer1">
                <circle
                    style="fill:#000000;stroke-width:0.204106"
                    id="path236"
                    cx="5.953125"
                    cy="5.953125"
                    r="3.5718751"
                ></circle>
                <ellipse
                    style="fill:#000000;stroke-width:0.238124"
                    id="path238"
                    cx="6.5719104"
                    cy="6.1122332"
                    rx="0.36909854"
                    ry="0.098487109"
                ></ellipse>
                <ellipse
                    style="fill:#000000;stroke-width:0.238124"
                    id="path348"
                    cx="6.8963585"
                    cy="5.628314"
                    rx="0.4582479"
                    ry="0.34205869"
                ></ellipse>
                <ellipse
                    style="fill:#000000;stroke-width:0.238124"
                    id="path350"
                    cx="5.1016603"
                    cy="4.344727"
                    rx="0.0076952553"
                    ry="0.033183888"
                ></ellipse>
                <rect
                    style="fill:#000000;stroke-width:0.254566"
                    id="rect428"
                    width="1.190625"
                    height="9.5249996"
                    x="5.3578124"
                    y="1.190625"
                ></rect>
                <rect
                    style="fill:#000000;stroke-width:0.254566"
                    id="rect428-3"
                    width="9.5249996"
                    height="1.190625"
                    x="1.190625"
                    y="5.3578124"
                ></rect>
                <rect
                    style="fill:#000000;stroke-width:0.254566"
                    id="rect428-5"
                    width="1.1906251"
                    height="9.5249996"
                    x="7.7933364"
                    y="-4.7618246"
                    transform="rotate(44.996916)"
                ></rect>
                <rect
                    style="fill:#000000;stroke-width:0.254566"
                    id="rect428-5-3"
                    width="1.1906251"
                    height="9.5249996"
                    x="-0.59553963"
                    y="3.6261489"
                    transform="matrix(-0.70714484,0.70706872,0.70706872,0.70714484,0,0)"
                ></rect>
            </g>
        </svg>
    }
}

#[component]
pub fn Star() -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            xmlns:xlink="http://www.w3.org/1999/xlink"
            viewBox="0 0 45 45"
            version="1.1"
        >
            <g id="surface1">
                <path
                    style="fill-rule:nonzero;fill:#fde047;fill-opacity:1;stroke-width:1;stroke-linecap:butt;stroke-linejoin:miter;stroke:#fde047;stroke-opacity:1;stroke-miterlimit:4;"
                    d="M 11.989583 2 C 6.470833 2 2 6.479167 2 12 C 2 17.520833 6.470833 22 11.989583 22 C 17.520833 22 22 17.520833 22 12 C 22 6.479167 17.520833 2 11.989583 2 Z M 16.229167 18 L 12 15.45 L 7.770833 18 L 8.889583 13.189583 L 5.160417 9.960417 L 10.079167 9.539583 L 12 5 L 13.920833 9.529167 L 18.839583 9.95 L 15.110417 13.179167 Z M 16.229167 18 "
                    transform="matrix(1.875,0,0,1.875,0,0)"
                ></path>
            </g>
        </svg>
    }
}

#[component]
pub fn Trophy() -> impl IntoView {
    view! {
        <svg viewBox="0 -0.5 26 26" fill="none" xmlns="http://www.w3.org/2000/svg">
            <rect
                x="11.9141"
                y="15.4102"
                width="1.58679"
                height="5.59554"
                fill="url(#paint0_linear_103_1804)"
            ></rect>
            <path
                d="M5.89393 3.5979H1C1 7.393 1.29104 9.57603 6.69821 9.57603"
                stroke="#FFDD66"
                stroke-width="2"
            ></path>
            <path
                d="M19.8636 8.56848C19.8636 12.5379 16.6458 15.7557 12.6764 15.7557C8.70707 15.7557 5.48926 12.5379 5.48926 8.56848C5.48926 4.59911 8.70707 1.3813 12.6764 1.3813C16.6458 1.3813 19.8636 4.59911 19.8636 8.56848Z"
                fill="#FFDD66"
            ></path>
            <path
                d="M12.6764 20.7262C9.74579 20.7262 7.37002 21.5833 7.37002 22.6406H17.9829C17.9829 21.5833 15.6071 20.7262 12.6764 20.7262Z"
                fill="#FFDD66"
            ></path>
            <path d="M5.48926 0H19.8636V8.23263H5.48926V0Z" fill="#FFDD66"></path>
            <path d="M17.9829 23.01H7.37002V22.607H17.9829V23.01Z" fill="#FFDD66"></path>
            <path
                d="M19.6603 3.5979H24.5542C24.5542 7.393 24.2632 9.57603 18.856 9.57603"
                stroke="#DE9300"
                stroke-width="2"
            ></path>
            <path
                d="M19.8634 8.56843C19.8634 12.5378 16.6456 15.7556 12.6762 15.7556C12.6762 15.7556 12.6762 12.5378 12.6762 8.56843C12.6762 4.59905 12.6762 1.38124 12.6762 1.38124C16.6456 1.38124 19.8634 4.59905 19.8634 8.56843Z"
                fill="url(#paint1_linear_103_1804)"
            ></path>
            <path
                d="M12.6762 20.7262C12.6762 20.7262 12.6762 21.5833 12.6762 22.6405H17.9826C17.9826 21.5833 15.6069 20.7262 12.6762 20.7262Z"
                fill="url(#paint2_linear_103_1804)"
            ></path>
            <path
                d="M12.6762 0.000488281H19.8634V8.23258H12.6762V0.000488281Z"
                fill="url(#paint3_linear_103_1804)"
            ></path>
            <path
                d="M17.9826 23.01H12.6762C12.6762 23.01 12.6643 22.7639 12.6762 22.6069C12.8331 20.5406 17.9826 22.6069 17.9826 22.6069V23.01Z"
                fill="url(#paint4_linear_103_1804)"
            ></path>
            <circle cx="12.8176" cy="7.76846" r="4.30105" fill="#DCAE0C"></circle>
            <circle
                cx="12.8088"
                cy="7.71544"
                r="3.12686"
                fill="#DE9300"
                stroke="#FFE176"
                stroke-width="4.55437"
            ></circle>
            <path
                d="M12.8087 4.17944L13.8984 6.35885L16.0778 6.63128L14.5812 8.30942L14.9881 10.7177L12.8087 9.62796L10.6293 10.7177L11.0397 8.30942L9.53955 6.63128L11.719 6.35885L12.8087 4.17944Z"
                fill="#FFF4BC"
            ></path>
            <path
                d="M13.2559 3.95584L12.8087 3.06141L12.3614 3.95584L11.3914 5.8959L9.47753 6.13514L8.53113 6.25344L9.16678 6.96451L10.5063 8.46298L10.1364 10.6337L9.97064 11.606L10.8529 11.1649L12.8087 10.187L14.7645 11.1649L15.6451 11.6052L15.4811 10.6344L15.1143 8.46295L16.4509 6.96406L17.0848 6.25327L16.1398 6.13514L14.2259 5.8959L13.2559 3.95584Z"
                stroke="#C98500"
                stroke-opacity="0.7"
            ></path>
            <rect x="5" y="23" width="15" height="2" fill="#DE9300"></rect>
            <defs>
                <linearGradient
                    id="paint0_linear_103_1804"
                    x1="12.7075"
                    y1="15.4102"
                    x2="12.7075"
                    y2="21.0057"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stop-color="#C07F00"></stop>
                    <stop offset="1" stop-color="#DE9300"></stop>
                </linearGradient>
                <linearGradient
                    id="paint1_linear_103_1804"
                    x1="19.8139"
                    y1="7.24836"
                    x2="12.6085"
                    y2="7.24836"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stop-color="#DE9300"></stop>
                    <stop offset="1" stop-color="#FFBC11"></stop>
                </linearGradient>
                <linearGradient
                    id="paint2_linear_103_1804"
                    x1="19.8139"
                    y1="7.24836"
                    x2="12.6085"
                    y2="7.24836"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stop-color="#DE9300"></stop>
                    <stop offset="1" stop-color="#FFBC11"></stop>
                </linearGradient>
                <linearGradient
                    id="paint3_linear_103_1804"
                    x1="19.8139"
                    y1="7.24836"
                    x2="12.6085"
                    y2="7.24836"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stop-color="#DE9300"></stop>
                    <stop offset="1" stop-color="#FFBC11"></stop>
                </linearGradient>
                <linearGradient
                    id="paint4_linear_103_1804"
                    x1="19.8139"
                    y1="7.24836"
                    x2="12.6085"
                    y2="7.24836"
                    gradientUnits="userSpaceOnUse"
                >
                    <stop stop-color="#DE9300"></stop>
                    <stop offset="1" stop-color="#FFBC11"></stop>
                </linearGradient>
            </defs>
        </svg>
    }
}
