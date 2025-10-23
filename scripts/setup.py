import json

if __name__ == "__main__":
    with open("./template.json", 'r') as config_file:
        config = json.load(config_file)
    print(config)
