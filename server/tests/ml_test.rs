use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;

#[test]
fn test_onnx_model_loading() {
    let model_path = "model.onnx";

    let session = Session::builder()
        .unwrap()
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .unwrap()
        .with_intra_threads(1)
        .unwrap()
        .commit_from_file(model_path);

    assert!(session.is_ok(), "Failed to load ONNX model");
}
