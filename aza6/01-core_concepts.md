# Core Concepts

## Chunk Dependencies and Order

Azadi-noweb allows you to write in a natural, top-down style. You can use chunks before defining them:

````markdown
```python
# <[@file src/app.py]>=
# src/app.py
from typing import Dict

# <[type_definitions]>
# <[core_functions]>
# <[main_loop]>
# $$

# <[type_definitions]>=
class Config(TypedDict):
    host: str
    port: int
    workers: int
# $$

# <[core_functions]>=
def load_config() -> Config:
    return {
        "host": "localhost",
        "port": 8080,
        "workers": 4
    }
# $$

# <[main_loop]>=
def main():
    config = load_config()
    print(f"Starting server on {config['host']}:{config['port']}")
# $$
```
````

## Chunk Indentation

Chunks maintain the indentation level where they're used:

````markdown
```python
# <[handler_code]>=
async def handle_request(request):
    response = await process(request)
    return response
# $$

# <[@file src/server.py]>=
# src/server.py
class Server:
    def __init__(self):
        self.routes = {}
        
    # <[handler_code]>
# $$
```
````

The `handle_request` function will be properly indented in the generated file.

## Multiple Uses

The same chunk can be used multiple times:

````markdown
```python
# <[error_handling]>=
try:
    result = operation()
except Exception as e:
    log.error(f"Failed: {e}")
    raise
# $$

# <[@file src/operations.py]>=
# src/operations.py
def first_operation():
    # <[error_handling]>
    
def second_operation():
    # <[error_handling]>
# $$
```
````

## Chunk Accumulation

Multiple definitions of the same chunk are concatenated:

````markdown
First we define basic logging:

```python
# <[setup_logging]>=
logging.basicConfig(level=logging.INFO)
# $$
```

Later we can add more configuration:

```python
# <[setup_logging]>=
logging.getLogger("requests").setLevel(logging.WARNING)
# $$
```

The final file will contain both parts:

```python
# <[@file src/logging_setup.py]>=
# src/logging_setup.py
import logging

# <[setup_logging]>
# $$
```
````
