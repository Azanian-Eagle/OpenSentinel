import numpy as np
from sklearn.ensemble import RandomForestClassifier
from sklearn.model_selection import train_test_split
from skl2onnx import convert_sklearn
from skl2onnx.common.data_types import FloatTensorType
import joblib

def generate_synthetic_data(num_samples=10000):
    """
    Generates sophisticated synthetic data to represent advanced human and bot behaviours.
    Features: avg_deviation, avg_speed, speed_variance, keystroke_interval, keystroke_variance
    """
    # 0 = Bot, 1 = Human
    X = []
    y = []

    for _ in range(num_samples):
        is_human = np.random.rand() > 0.5

        if is_human:
            # Humans: Curved paths (high deviation), variable speed, hesitation (high speed variance), variable typing
            avg_deviation = np.random.uniform(0.1, 1.5)
            avg_speed = np.random.uniform(10.0, 100.0)
            speed_variance = np.random.uniform(20.0, 300.0)

            keystroke_interval = np.random.uniform(100.0, 400.0)
            keystroke_variance = np.random.uniform(30.0, 150.0)

            y.append(1)
        else:
            # Bots:
            # Type 1: Linear (low deviation), constant speed
            # Type 2: Ultra-fast teleportation (extreme speed, low variance)
            # Type 3: Scripted repetitive clicking (extreme low variance)
            bot_type = np.random.choice([1, 2, 3])

            if bot_type == 1: # Linear script
                avg_deviation = np.random.uniform(0.0, 0.05)
                avg_speed = np.random.uniform(5.0, 200.0)
                speed_variance = np.random.uniform(0.0, 5.0)
                keystroke_interval = np.random.uniform(50.0, 100.0)
                keystroke_variance = np.random.uniform(0.0, 2.0)
            elif bot_type == 2: # Teleportation
                avg_deviation = np.random.uniform(0.0, 0.5)
                avg_speed = np.random.uniform(500.0, 2000.0)
                speed_variance = np.random.uniform(0.0, 10.0)
                keystroke_interval = np.random.uniform(10.0, 30.0)
                keystroke_variance = np.random.uniform(0.0, 1.0)
            else: # Macro playback (exact repetition)
                avg_deviation = np.random.uniform(0.05, 0.2)
                avg_speed = np.random.uniform(30.0, 60.0)
                speed_variance = 0.0
                keystroke_interval = 200.0
                keystroke_variance = 0.0

            y.append(0)

        X.append([avg_deviation, avg_speed, speed_variance, keystroke_interval, keystroke_variance])

    return np.array(X), np.array(y)

if __name__ == "__main__":
    print("Generating sophisticated synthetic telemetry data...")
    X, y = generate_synthetic_data(20000)

    print("Splitting data...")
    X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.2, random_state=42)

    print("Training Next-Gen Random Forest model...")
    clf = RandomForestClassifier(n_estimators=100, max_depth=10, random_state=42)
    clf.fit(X_train, y_train)

    accuracy = clf.score(X_test, y_test)
    print(f"Model accuracy on synthetic test set: {accuracy * 100:.2f}%")

    # Export to ONNX
    print("Exporting model to ONNX format without zipmap (for rust ndarray compatibility)...")
    initial_type = [('float_input', FloatTensorType([None, 5]))]
    options = {id(clf): {'zipmap': False}}
    onx = convert_sklearn(clf, initial_types=initial_type, options=options)

    with open("server/model.onnx", "wb") as f:
        f.write(onx.SerializeToString())

    print("Exported to server/model.onnx successfully. Phase 2 Retraining Complete.")
