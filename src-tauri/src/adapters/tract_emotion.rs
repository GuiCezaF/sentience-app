use crate::agent::{EmotionModel, FaceCrop, PortError, CROP_SIDE};
use std::path::Path;
use tract_onnx::prelude::*;

pub struct TractEmotionModel {
    model: Arc<TypedRunnableModel>,
}

impl TractEmotionModel {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, PortError> {
        let model = tract_onnx::onnx()
            .model_for_path(path.as_ref())
            .map_err(tract_err)?
            .with_input_fact(
                0,
                u8::fact([1, CROP_SIDE as i64, CROP_SIDE as i64, 1]).into(),
            )
            .map_err(tract_err)?
            .into_optimized()
            .map_err(tract_err)?
            .into_runnable()
            .map_err(tract_err)?;
        Ok(Self { model })
    }
}

impl EmotionModel for TractEmotionModel {
    fn classify(&self, crop: &FaceCrop) -> Result<[f32; 4], PortError> {
        let tensor = crop_to_tensor(crop)?;
        let outputs = self.model.run(tvec!(tensor.into())).map_err(tract_err)?;
        probabilities_from_outputs(&outputs)
    }
}

fn crop_to_tensor(crop: &FaceCrop) -> Result<Tensor, PortError> {
    Tensor::from_shape(
        &[1, CROP_SIDE as usize, CROP_SIDE as usize, 1],
        &crop.pixels,
    )
    .map_err(tract_err)
}

fn probabilities_from_outputs(outputs: &[TValue]) -> Result<[f32; 4], PortError> {
    for output in outputs {
        let view = output.to_plain_array_view::<f32>().map_err(tract_err)?;
        let data = view
            .as_slice()
            .ok_or_else(|| PortError::new("DS-CNN: saída não contígua"))?;
        match view.shape() {
            [1, 4] | [4] if data.len() >= 4 => {
                return Ok([data[0], data[1], data[2], data[3]]);
            }
            _ => {}
        }
    }
    Err(PortError::new("DS-CNN: saída probabilities [1,4] ausente"))
}

fn tract_err(err: impl std::fmt::Display) -> PortError {
    PortError::new(format!("DS-CNN: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::CROP_PIXELS;

    #[test]
    fn crop_to_tensor_copia_bytes_sem_normalizar() {
        let mut pixels = [0u8; CROP_PIXELS];
        pixels[0] = 0;
        pixels[1] = 127;
        pixels[2] = 255;
        let crop = FaceCrop { pixels };

        let tensor = crop_to_tensor(&crop).expect("tensor");
        assert_eq!(
            tensor.shape(),
            &[1, CROP_SIDE as usize, CROP_SIDE as usize, 1]
        );
        let view = tensor.to_plain_array_view::<u8>().expect("u8");
        let data = view.as_slice().expect("contíguo");
        assert_eq!(&data[..3], &[0, 127, 255]);
        assert_eq!(data.len(), CROP_PIXELS);
    }

    #[test]
    fn probabilities_exige_quatro_valores() {
        let empty: [TValue; 0] = [];
        assert!(probabilities_from_outputs(&empty).is_err());
    }

    #[test]
    fn load_arquivo_ausente_falha() {
        assert!(TractEmotionModel::load("nao-existe.onnx").is_err());
    }
}
