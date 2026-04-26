use ndarray::Array2;
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::Tensor;

#[test]
fn test_bot_prediction() {
    let model_path = "model.onnx";

    let mut session = Session::builder()
        .unwrap()
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .unwrap()
        .with_intra_threads(1)
        .unwrap()
        .commit_from_file(model_path)
        .expect("Failed to load ONNX model");

    {
        // Test a bot input (very straight, fast, low variance)
        let input_array =
            Array2::<f32>::from_shape_vec((1, 5), vec![0.1, 8.0, 0.00001, 20.0, 1.0]).unwrap();
        let tensor = Tensor::from_array(input_array).unwrap();

        let outputs = session
            .run(ort::inputs!["float_input" => tensor.view()])
            .unwrap();
        let label_output = &outputs[0];
        let (_, data) = label_output.try_extract_tensor::<i64>().unwrap();
        let label = data[0];
        assert_eq!(label, 0); // Bot
    }

    {
        // Test a human input (not straight, normal speed, higher variance)
        let input_array2 =
            Array2::<f32>::from_shape_vec((1, 5), vec![2.5, 2.0, 0.5, 150.0, 50.0]).unwrap();
        let tensor2 = Tensor::from_array(input_array2).unwrap();

        let outputs2 = session
            .run(ort::inputs!["float_input" => tensor2.view()])
            .unwrap();
        let label_output2 = &outputs2[0];
        let (_, data2) = label_output2.try_extract_tensor::<i64>().unwrap();
        let label2 = data2[0];
        assert_eq!(label2, 1); // Human
    }
}
