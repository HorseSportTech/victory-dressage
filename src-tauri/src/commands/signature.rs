use hypertext::{rsx, rsx_static, GlobalAttributes, Renderable};
use serde::{Deserialize, Serialize};
use tauri::Manager;
use tauri_plugin_store::StoreExt;

use super::{
    fetch::{fetch, Method},
    replace_director::{ReplaceDirector, ResponseDirector},
};
use crate::{
    commands::replace_director::PageLocation,
    debug,
    templates::{html_elements, preferences::signatures_path},
};
use crate::{
    state::{ManagedApplicationState, UserType},
    templates::error::screen_error,
    traits::{Entity, Storable},
    STORE_URI,
};

const DEFAULT_WIDTH: f32 = 190.0;
const DEFAULT_HEIGHT: f32 = 90.0;

#[tauri::command]
pub async fn draw_signature(
    handle: tauri::AppHandle,
    point_lists: Vec<Vec<Point>>,
) -> Result<String, String> {
    let mut path = String::new();
    let point_lists = scale_points(point_lists);
    for points in point_lists {
        path.push_str(&convert_to_quatratic_bezier(points));
    }
    handle
        .store(STORE_URI)
        .map_err(|e| e.to_string())?
        .set("temp-signature", serde_json::Value::String(path.clone()));
    Ok(path)
}
#[tauri::command]
pub async fn save_signature(handle: tauri::AppHandle) -> ResponseDirector {
    let store = handle
        .store(STORE_URI)
        .map_err(|e| screen_error(e.to_string().as_str()))?;
    let signature: String = match store.get("temp-signature") {
        Some(s) => serde_json::from_value(s).expect("Should be able to parse to string"),
        None => return Ok(ReplaceDirector::with_target(&PageLocation::SignatureDialogMessage, rsx!{
            <div style="background:red; color:white; font-weight:bold; corner-radius:var(--corner-size)">
            "Signature not found"</div>
        }.render()))
    };
    store.delete("temp-signature");

    let state = handle.state::<ManagedApplicationState>();

    let id = {
        let Some(id) = state
            .read_async(|app_state| match app_state.user {
                UserType::Judge(ref judge, _) => Some(judge.get_id()),
                _ => None,
            })
            .await?
        else {
            return crate::commands::log_out::log_out(state.clone(), handle.clone()).await;
        };
        id
    };

    let json = format!("{{\"signature\": \"{signature}\"}}");
    let _ = fetch(Method::Put, &format!(concat!(env!("API_URL"), "judge/{}"), &id), &state)
        .body(json)
        .send().await
        .map_err(|err| {
            debug!("{err:?}");
            ReplaceDirector::with_target(
                &PageLocation::SignatureDialogMessage,
                rsx_static!{<div style="background:red; color:white; font-weight:bold; corner-radius:var(--corner-size)">
                "Could not save signature"</div>}.render())
        })?
        .error_for_status()
        .map_err(|err| {
            debug!("{err:?}");
            ReplaceDirector::with_target(
                &PageLocation::SignatureDialogMessage,
                rsx_static!{<div style="background:red; color:white; font-weight:bold; corner-radius:var(--corner-size)">
                "Could not save signature"</div>}.render())
        })?
        .text().await
        .map_err(|err| {
            debug!("{err:?}");
            ReplaceDirector::with_target(
                &PageLocation::SignatureDialogMessage,
                rsx_static!{<div style="background:red; color:white; font-weight:bold; corner-radius:var(--corner-size)">
                "Could not save signature"</div>}.render())
        })?;

    let handle2 = handle.clone();
    let signature2 = signature.clone();
    state
        .write_async(move |app_state| {
            if let UserType::Judge(ref mut judge, _) = app_state.user {
                judge.signature = Some(signature2);
            };
            app_state.clone().set(&handle2).ok();
        })
        .await?;

    Ok(ReplaceDirector::with_target(
        &PageLocation::SignatureImage,
        signatures_path(&Some(signature)).render(),
    ))
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Point {
    x: f32,
    y: f32,
}

fn scale_points(mut point_lists: Vec<Vec<Point>>) -> Vec<Vec<Point>> {
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;

    for points in &point_lists {
        for point in points {
            if point.x < min_x {
                min_x = point.x
            }
            if point.x > max_x {
                max_x = point.x
            }
            if point.y < min_y {
                min_y = point.y
            }
            if point.y > max_y {
                max_y = point.y
            }
        }
    }

    // we make this slightly smaller than the ideal size to account for the stroke width
    let scale_x = DEFAULT_WIDTH / (max_x - min_x) * 0.95;
    let scale_y = DEFAULT_HEIGHT / (max_y - min_y) * 0.95;
    let scale = f32::min(scale_x, scale_y);

    let height = (max_y - min_y) * scale;
    let width = (max_x - min_x) * scale;

    // we deduct the offset from the scaled size to center the signature
    let scaled_offset_y = (DEFAULT_HEIGHT - height) / 2.0;
    let scaled_offset_x = (DEFAULT_WIDTH - width) / 2.0;

    for points in point_lists.iter_mut() {
        for point in points.iter_mut() {
            point.x = ((point.x - min_x) * scale) + scaled_offset_x;
            point.y = ((point.y - min_y) * scale) + scaled_offset_y;
        }
    }
    point_lists
}

fn convert_to_quatratic_bezier(points: Vec<Point>) -> String {
    let simplified_points = douglas_peucker(&points[..], 2.0);
    generate_path_data(simplified_points)
}

fn generate_path_data(simplified_points: Vec<Point>) -> String {
    if simplified_points.is_empty() {
        return "".into(); // No points to create a curve
    } else if simplified_points.len() < 3 {
        return format!(
            "M{} {}L{} {}",
            sig_round(
                simplified_points
                    .first()
                    .expect("We know this exists as we checked it above")
                    .x,
                1
            ),
            sig_round(
                simplified_points
                    .first()
                    .expect("We know this exists as we checked it above")
                    .y,
                1
            ),
            sig_round(
                simplified_points
                    .last()
                    .expect("We know this exists as we checked it above")
                    .x,
                1
            ),
            sig_round(
                simplified_points
                    .last()
                    .expect("We know this exists as we checked it above")
                    .y,
                1
            ),
        );
    }

    let mut path = format!(
        "M{} {}",
        sig_round(simplified_points[0].x, 1),
        sig_round(simplified_points[0].y, 1),
    );
    for i in 1..simplified_points.len() {
        let cp = simplified_points[i];
        let next = simplified_points[if i + 1 > (simplified_points.len() - 1) {
            i
        } else {
            i + 1
        }];
        path = format!(
            "{}Q{} {},{} {}",
            path,
            sig_round(cp.x, 1),
            sig_round(cp.y, 1),
            sig_round((cp.x + next.x) / 2.0, 1),
            sig_round((cp.y + next.y) / 2.0, 1),
        );
    }
    format!(
        "{}L{} {}",
        path,
        sig_round(simplified_points.last().unwrap().x, 1),
        sig_round(simplified_points.last().unwrap().y, 1)
    )
}

fn douglas_peucker(points: &[Point], epsilon: f32) -> Vec<Point> {
    // Find the point with the maximum distance from line
    let mut max_distance = 0.0;
    let mut index: usize = 0;

    let first_point = points
        .first()
        .expect("We know this exists as we checked it above");
    let last_point = points
        .last()
        .expect("We know this exists as we checked it above");
    for (idx, current_point) in points.iter().enumerate().skip(1) {
        let distance = perpendicular_distance(current_point, first_point, last_point);
        if distance <= max_distance {
            continue;
        }
        index = idx;
        max_distance = distance;
    }

    // If max distance is greater than epsilon, recursively simplify
    if max_distance > epsilon {
        let rec_results1 = douglas_peucker(&points[0..index + 1], epsilon);
        let rec_results2 = douglas_peucker(&points[index..points.len()], epsilon);

        // Build the result list
        [&rec_results1[0..rec_results1.len() - 1], &rec_results2[..]]
            .concat()
            .to_owned()
    } else {
        vec![*first_point, *last_point]
    }
}

fn perpendicular_distance(point: &Point, line_start: &Point, line_end: &Point) -> f32 {
    let dx = line_end.x - line_start.x;
    let dy = line_end.y - line_start.y;

    let numerator =
        (dy * point.x - dx * point.y + line_end.x * line_start.y - line_end.y * line_start.x).abs();
    let denominator = (dx * dx + dy * dy).sqrt();

    numerator / denominator
}

fn sig_round(x: f32, decimals: u32) -> String {
    if x == 0. || decimals == 0 {
        "0".into()
    } else {
        let shift_factor = 10_f64.powi(decimals as i32);

        let number = (x as f64 * shift_factor).round() / shift_factor;
        if number == number.trunc() {
            format!("{number}")
        } else {
            format!("{number:.1}")
        }
    }
}
