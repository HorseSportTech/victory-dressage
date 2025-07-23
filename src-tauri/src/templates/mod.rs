#![allow(non_upper_case_globals)]
use hypertext::{Attribute, GlobalAttributes};

pub mod choose_judge;
pub mod competition_list;
pub mod error;
pub mod icons;
pub mod login;
pub mod logout;
pub mod preferences;
pub mod result;
pub mod scoresheet;
pub mod settings;
pub mod welcome;

pub mod html_elements {
    pub use hypertext::html_elements::*;
    hypertext::elements! {
        svg {
            viewBox
            width
            height
            preserveAspectRatio
        }
        g {
            transform
        }
        text {
            x
            y
            font_size
            transform_origin
            font_weight
            text_anchor
        }
        path {
            d
            fill
            stroke
            stroke_width
        }
    }
}

#[allow(dead_code)]
pub trait TxAttributes: GlobalAttributes {
    const tx_open: Attribute = Attribute;
    const tx_goto: Attribute = Attribute;
    const tx_command: Attribute = Attribute;
    const tx_id: Attribute = Attribute;
    const tx_data: Attribute = Attribute;
    const tx_trigger: Attribute = Attribute;
    const data_active: Attribute = Attribute;
    const data_row_type: Attribute = Attribute;
    const data_attempt_index: Attribute = Attribute;
    const data_input_role: Attribute = Attribute;
    const data_exercise_comment_last: Attribute = Attribute;
    const onclick: Attribute = Attribute;
    const onkeyup: Attribute = Attribute;
    const onkeydown: Attribute = Attribute;
    const onload: Attribute = Attribute;
    const oninput: Attribute = Attribute;
    const onblur: Attribute = Attribute;
    const onbeforeinput: Attribute = Attribute;
    const value: Attribute = Attribute;
    const onpointerdown: Attribute = Attribute;
    const onpointermove: Attribute = Attribute;
    const onpointerup: Attribute = Attribute;
    const onpointerleave: Attribute = Attribute;
}
impl<T: GlobalAttributes> TxAttributes for T {}
