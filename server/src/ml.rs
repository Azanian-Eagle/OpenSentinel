use ndarray::Array2;
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::Tensor;
use std::env;
use std::sync::Mutex;
use std::sync::OnceLock;

static SESSION: OnceLock<Result<Mutex<Session>, String>> = OnceLock::new();

pub fn model_path() -> String {
    env::var("OPEN_SENTINEL_MODEL_PATH")
        .or_else(|_| env::var("MODEL_PATH"))
        .unwrap_or_else(|_| "model.onnx".to_string())
}

pub fn get_session() -> Result<&'static Mutex<Session>, String> {
    SESSION
        .get_or_init(|| {
            let model_path = model_path();

        Session::builder()
            .unwrap()
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .unwrap()
            .with_intra_threads(1)
            .unwrap()
            .commit_from_file(&model_path)
            .map(Mutex::new)
            .map_err(|error| format!("Failed to load ONNX model at {}: {}", model_path, error))
        })
        .as_ref()
        .map_err(|error| error.clone())
}

pub fn predict_bot_probability(
    avg_deviation: f64,
    avg_speed: f64,
    speed_variance: f64,
    keystroke_interval: f64,
    keystroke_variance: f64,
) -> Result<f64, Box<dyn std::error::Error>> {
    let session = get_session().map_err(std::io::Error::other)?;
    let mut session = session.lock().unwrap();

    let input_array = Array2::<f32>::from_shape_vec(
        (1, 5),
        vec![
            avg_deviation as f32,
            avg_speed as f32,
            speed_variance as f32,
            keystroke_interval as f32,
            keystroke_variance as f32,
        ],
    )?;

    // We must use Tensor::from_array to properly wrap ndarray types into ONNX value
    let tensor = Tensor::from_array(input_array)?;

    let outputs = session.run(ort::inputs!["float_input" => tensor.view()])?;

    let label_output = &outputs[0];
    let (_, data) = label_output.try_extract_tensor::<i64>()?;
    let label = data[0];

    Ok(if label == 1 { 1.0 } else { 0.0 })
}
