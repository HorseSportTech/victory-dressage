use crate::templates::html_elements;
use hypertext::{rsx_static, Raw};

pub const BACK_ARROW: Raw<&'static str> = rsx_static! {
<svg viewBox="0 0 2 2"><path d="M0,1L2,0V2z" fill="currentColor"></path></svg>
};

pub const TRASH: Raw<&'static str> = rsx_static! {
<svg viewBox="0 0 20 20"><path d="M10,0S20,0 17,3 H20V5H17V20H3V5H0V3H3 S2,0 10,0z M7,5V18H9V5z M12,5V18H14V5z" fill="currentColor"></path></svg>
};

pub const CLOSE: Raw<&'static str> = rsx_static! {
<svg viewBox="0 0 20 20"><path d="M2,0L10,8L18,0L20,2L12,10L20,18L18,20L10,12L2,20L0,18L8,10L0,2z" fill="currentColor"></path></svg>
};

pub const TICK: Raw<&'static str> = rsx_static! {
<svg viewBox="0 0 20 20"><path d="M2,10L7,15L18,1L20,3L7.5,18.5L0,12z" fill="currentColor"></path></svg>
};

pub const EDIT: Raw<&'static str> = rsx_static! {
<svg viewBox="0 0 20 20"><path d="M0,20L1,15L16,0S20,0 20,4L5,19zM1,19L5,18L2,15z" fill="currentColor"></path></svg>
};

pub const PAUSE: Raw<&'static str> = rsx_static! {
<svg viewBox="0 0 20 20"><path d="M2,0V20H8V0zM12,0V20H18V0z" fill="currentColor"></path></svg>
};

pub const WARNING: Raw<&'static str> = rsx_static! {
<svg viewBox="0 0 36 36">
    <path fill="currentColor" d="M18,0L36,35H0 M18,6.5L5,32H31z M16,31H20V28H16z M16,26H20L21,12H15z"></path>
</svg>
};

pub const MENU: Raw<&'static str> = rsx_static! {
<svg viewBox="0 0 5 5"><path d="M0,0H5v1H0zM0,2H5v1H0zM0,4H5v1H0z" fill="currentColor"></path></svg>
};
