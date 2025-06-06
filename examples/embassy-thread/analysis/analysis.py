import sys
import re
import csv
import os
from collections import defaultdict
from statistics import mean, stdev
from datetime import datetime

LOG_PATTERNS = {
    "async_report": {
        "regex": r'ASYNC_TASK_REPORT (\d+): Iteration (\d+), expected (\d+)/ns, actual (\d+)/ns, full (\d+)/ns',
        "id_key": "task_id",
        "metrics": ["actual_ns", "full_ns"]
    },
    "native_report": {
        "regex": r'NATIVE_THREAD_REPORT (\d+): Iteration (\d+), expected (\d+)/ns, actual (\d+)/ns, full (\d+)/ns',
        "id_key": "thread_id",
        "metrics": ["actual_ns", "full_ns"]
    }
}

CSV_HEADERS = [
    "timestamp",
    "log_type",
    "metric_name", 
    "count",
    "mean_absolute_deviation",
    "std_deviation",
    "min_deviation",
    "max_deviation"
]

def write_to_csv(deviation_data, log_path, csv_path):
    timestamp = datetime.now().isoformat()
    log_filename = os.path.basename(log_path)
    
    file_exists = os.path.exists(csv_path)
    
    try:
        with open(csv_path, 'a', newline='') as csvfile:
            writer = csv.writer(csvfile)
            
            # Write headers if file is new
            if not file_exists:
                writer.writerow(CSV_HEADERS)
            
            # Write data rows
            for log_type, metrics in deviation_data.items():
                for metric_name, deviations in metrics.items():
                    if deviations:  # Only write if we have data
                        std_dev = stdev(deviations) if len(deviations) > 1 else 0
                        
                        row = [
                            timestamp,
                            log_type,
                            metric_name,
                            len(deviations),
							round(mean(deviations),2),
                            round(std_dev,2),
                            min(deviations),
                            max(deviations)
                        ]

                        writer.writerow(row)
                        
    except Exception as e:
        print(f"Error writing to CSV file '{csv_file_path}': {e}", file=sys.stderr)
        sys.exit(1)

def parse_log_file(path):
    parsed_data = defaultdict(lambda: defaultdict(list))

    patterns = {name: re.compile(config["regex"]) for name, config in LOG_PATTERNS.items()} 
    try:
        with open(path, "r") as f:
            for line in f:
                for log_type, config in LOG_PATTERNS.items():
                    match = patterns[log_type].search(line)
                    if match:
                        _id, _, expected, *metrics = map(int, match.groups())
                        for key, value in zip(config["metrics"], metrics):
                            deviation = abs(value - expected)
                            parsed_data[log_type][key].append(deviation)
                        break
    except FileNotFoundError:
        print(f"Error: Log file '{path}' not found.", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"An error occurred while parsing the log file: {e}", file=sys.stderr)
        sys.exit(1)

    return parsed_data

def summarize_deviation(deviation_data):
    for log_type, metrics in deviation_data.items():
        print(f"--- {log_type} ---")
        for metric_name, deviations in metrics.items():
            print(f"  Metric: {metric_name}")
            print(f"    Count: {len(deviations)}")
            print(f"    Mean Absolute Deviation: {mean(deviations):.2f} ns")
            print(f"    Std Dev: {stdev(deviations):.2f} ns" if len(deviations) > 1 else "    Std Dev: N/A")
            print(f"    Min: {min(deviations)} ns")
            print(f"    Max: {max(deviations)} ns")
        print()

def main():
    if len(sys.argv) not in [2, 3]:
        print("Usage: python3 deviation_summary.py <logfile> [output.csv]", file=sys.stderr)
        print("If output.csv is not specified, defaults to 'deviation_summary.csv'", file=sys.stderr)
        sys.exit(1)

    log_file_path = sys.argv[1]
    csv_file_path = sys.argv[2] if len(sys.argv) == 3 else "summary.csv"

    try:
        deviations = parse_log_file(sys.argv[1])

        if not any(deviations.values()):
            print("No matching log entries found in the file.", file=sys.stderr)
            sys.exit(1)
            
        write_to_csv(deviations, log_file_path, csv_file_path)
        
        summarize_deviation(deviations)
    except FileNotFoundError:
        print(f"Error: File '{sys.argv[1]}' not found.", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()