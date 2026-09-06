use crate::agent::{FaceBox, FaceFinder, Frame, PortError};
use image::imageops::{resize, FilterType};
use image::RgbImage;
use std::path::Path;
use tract_onnx::prelude::*;

const INPUT_WIDTH: u32 = 320;
const INPUT_HEIGHT: u32 = 240;
const SCORE_FACE_THRESHOLD: f32 = 0.7;
const PIXEL_MEAN: f32 = 127.0;
const PIXEL_SCALE: f32 = 128.0;

pub struct UltraFaceFinder {
    model: Arc<TypedRunnableModel>,
}

impl UltraFaceFinder {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, PortError> {
        let model = tract_onnx::onnx()
            .model_for_path(path.as_ref())
            .map_err(tract_err)?
            .with_input_fact(0, f32::fact([1, 3, INPUT_HEIGHT as i64, INPUT_WIDTH as i64]).into())
            .map_err(tract_err)?
            .into_optimized()
            .map_err(tract_err)?
            .into_runnable()
            .map_err(tract_err)?;
        Ok(Self { model })
    }
}

impl FaceFinder for UltraFaceFinder {
    fn detect(&self, frame: &Frame) -> Result<Vec<FaceBox>, PortError> {
        let (frame_w, frame_h) = frame.size();
        if frame_w == 0 || frame_h == 0 {
            return Ok(Vec::new());
        }

        let tensor = preprocess(frame.as_rgb())?;
        let outputs = self.model.run(tvec!(tensor.into())).map_err(tract_err)?;
        let (scores, boxes, anchors) = scores_and_boxes(&outputs)?;
        Ok(faces_from_tensors(&scores, &boxes, anchors, frame_w, frame_h))
    }
}

fn preprocess(rgb: &RgbImage) -> Result<Tensor, PortError> {
    let resized = resize(rgb, INPUT_WIDTH, INPUT_HEIGHT, FilterType::Triangle);
    let nchw = rgb_to_nchw(&resized);
    Tensor::from_shape(
        &[1, 3, INPUT_HEIGHT as usize, INPUT_WIDTH as usize],
        &nchw,
    )
    .map_err(tract_err)
}

fn rgb_to_nchw(image: &RgbImage) -> Vec<f32> {
    let (width, height) = image.dimensions();
    let w = width as usize;
    let h = height as usize;
    let plane = w * h;
    let raw = image.as_raw();
    let mut nchw = vec![0.0f32; 3 * plane];
    for y in 0..h {
        for x in 0..w {
            let src = (y * w + x) * 3;
            let dst = y * w + x;
            nchw[dst] = (raw[src] as f32 - PIXEL_MEAN) / PIXEL_SCALE;
            nchw[plane + dst] = (raw[src + 1] as f32 - PIXEL_MEAN) / PIXEL_SCALE;
            nchw[2 * plane + dst] = (raw[src + 2] as f32 - PIXEL_MEAN) / PIXEL_SCALE;
        }
    }
    nchw
}

fn scores_and_boxes(outputs: &[TValue]) -> Result<(Vec<f32>, Vec<f32>, usize), PortError> {
    let mut scores = None;
    let mut boxes = None;

    for output in outputs {
        let view = output.to_plain_array_view::<f32>().map_err(tract_err)?;
        let data = view
            .as_slice()
            .ok_or_else(|| PortError::new("UltraFace: saída não contígua"))?;
        match view.shape() {
            [1, n, 2] | [n, 2] => scores = Some((data.to_vec(), *n)),
            [1, n, 4] | [n, 4] => boxes = Some((data.to_vec(), *n)),
            _ => {}
        }
    }

    let (scores, n_scores) =
        scores.ok_or_else(|| PortError::new("UltraFace: saída scores [..,2] ausente"))?;
    let (boxes, n_boxes) =
        boxes.ok_or_else(|| PortError::new("UltraFace: saída boxes [..,4] ausente"))?;
    if n_scores != n_boxes {
        return Err(PortError::new(format!(
            "UltraFace: âncoras divergentes scores={n_scores} boxes={n_boxes}"
        )));
    }
    if scores.len() < n_scores * 2 || boxes.len() < n_boxes * 4 {
        return Err(PortError::new("UltraFace: tensor menor que o shape declarado"));
    }
    Ok((scores, boxes, n_scores))
}

fn faces_from_tensors(
    scores: &[f32],
    boxes: &[f32],
    anchors: usize,
    frame_w: u32,
    frame_h: u32,
) -> Vec<FaceBox> {
    let mut faces = Vec::new();
    for i in 0..anchors {
        // scores[..., 0] = fundo; scores[..., 1] = face
        if scores[i * 2 + 1] < SCORE_FACE_THRESHOLD {
            continue;
        }
        let base = i * 4;
        if let Some(face) = normalized_to_face_box(
            boxes[base],
            boxes[base + 1],
            boxes[base + 2],
            boxes[base + 3],
            frame_w,
            frame_h,
        ) {
            faces.push(face);
        }
    }
    faces
}

fn normalized_to_face_box(
    xmin: f32,
    ymin: f32,
    xmax: f32,
    ymax: f32,
    frame_w: u32,
    frame_h: u32,
) -> Option<FaceBox> {
    if frame_w == 0 || frame_h == 0 {
        return None;
    }
    let fw = frame_w as f32;
    let fh = frame_h as f32;
    let x1 = (xmin.min(xmax) * fw).clamp(0.0, fw);
    let y1 = (ymin.min(ymax) * fh).clamp(0.0, fh);
    let x2 = (xmin.max(xmax) * fw).clamp(0.0, fw);
    let y2 = (ymax.max(ymin) * fh).clamp(0.0, fh);

    let x = x1.floor() as u32;
    let y = y1.floor() as u32;
    if x >= frame_w || y >= frame_h {
        return None;
    }
    let w = (x2.ceil() as u32).min(frame_w).saturating_sub(x);
    let h = (y2.ceil() as u32).min(frame_h).saturating_sub(y);
    if w == 0 || h == 0 {
        return None;
    }
    Some(FaceBox { x, y, w, h })
}

fn tract_err(err: impl std::fmt::Display) -> PortError {
    PortError::new(format!("UltraFace: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgb;

    #[test]
    fn nchw_normaliza_e_separa_canais() {
        let img = RgbImage::from_pixel(1, 1, Rgb([255, 127, 0]));
        let nchw = rgb_to_nchw(&img);
        assert_eq!(nchw.len(), 3);
        assert!((nchw[0] - (255.0 - 127.0) / 128.0).abs() < 1e-6);
        assert!(nchw[1].abs() < 1e-6);
        assert!((nchw[2] - (0.0 - 127.0) / 128.0).abs() < 1e-6);
    }

    #[test]
    fn caixa_normalizada_vira_pixels_do_frame() {
        let face = normalized_to_face_box(0.25, 0.25, 0.75, 0.75, 320, 240).unwrap();
        assert_eq!(
            face,
            FaceBox {
                x: 80,
                y: 60,
                w: 160,
                h: 120
            }
        );
    }

    #[test]
    fn score_abaixo_do_limiar_e_ignorado() {
        let scores = [0.9, 0.69, 0.1, 0.70];
        let boxes = [
            0.0, 0.0, 1.0, 1.0, 0.1, 0.1, 0.2, 0.2,
        ];
        let faces = faces_from_tensors(&scores, &boxes, 2, 100, 100);
        assert_eq!(faces.len(), 1);
        assert_eq!(
            faces[0],
            FaceBox {
                x: 10,
                y: 10,
                w: 10,
                h: 10
            }
        );
    }
}
