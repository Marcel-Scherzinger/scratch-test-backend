import requests
import zipfile
import json


def main(program_path, spec):
    with zipfile.ZipFile(program_path) as f:
        program = f.read("project.json")

    program = json.loads(program)

    resp = requests.post(
        "http://localhost:42139/api/v1/run",
        headers={"Content-Type": "application/json"},
        json={"specification": spec, "program": program},
    )
    print(resp)


if __name__ == "__main__":
    program_path = "../../sb3/sum-from-to-with-repeat-loop.sb3"
    spec = {
        "lints": [
            {
                "severity": "error",
                "condition": {
                    "type": "block-count-limit",
                    "opcode": "control_repeat",
                    "max": 0,
                },
            }
        ],
        "categories": [
            {
                "type": "static",
                "cases": [
                    {
                        "inputs": ["1", "10"],
                        "checks": [
                            {
                                "severity": "error",
                                "condition": {
                                    "select": {"type": "last-line"},
                                    "transformations": [
                                        {"action": "extract-single-number"}
                                    ],
                                    "criterion": {"type": "equal-texts", "other": "45"},
                                },
                            }
                        ],
                    }
                ],
            }
        ],
    }
    main(program_path, spec)
