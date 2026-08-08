<h1 align="center">SMLOG</h1>

<p align="center">
  <a href="https://training.linuxfoundation.org">
    <img src="https://img.shields.io/badge/Linux-supported-green?logo=linux">
  </a>
  <a href="https://www.apple.com/id/os/macos/">
    <img src="https://img.shields.io/badge/MacOS-supported-green?logo=darwin">
  </a>
  <a href="https://www.microsoft.com/en-us/windows">
    <img src="https://img.shields.io/badge/Windows-supported-green?logo=windows">
  </a>
</p>


**smlog** is a high-performance logging library for Python rewritten using Rust and PyO3. Serves as a drop-in extension for `print()` Python defaults, `smlog` offers much lower overhead, massive log throughput handling, and separation of log execution between output terminals and persistent SQLite storage.

---

## Key Features & Architecture

**Zero-Lag Terminal I/O (`sml.printf`):** Replaces Python's built-in I/O mechanism with Rust FFI bindings optimized for executing large logs without triggering I/O bottlenecks.

**Silent SQLite Storage (`sml.printd`):** Isolates debugging logs and error tracebacks directly to a structured SQLite database in the OS `/tmp` directory without filling up the terminal stdout buffer.

**Native Type Ingestion:** The FFI layer handles Python data type conversion to Rust strings directly (`PyBytes`, `NoneType`, and custom classes via slots `__str__`).

**Python Print Compatible:** Supporting conventional arguments such as `sep`, `end`, `file`, And `flush`.

---

## Technical Performance Highlights

1. **Overhead & Speed:** Reduces I/O interrupt overhead on massive log execution by moving the formatting and text writing process to the Rust native runtime.
2. **Crash & Traceback Capture:** `sml.printd` automatically extracts stack traces and variable metadata when catching exceptions, saving them to a structured SQLite table.

---

## Installation

```bash
# Pip install via wheel binary (Rust Toolchain required if building from source)
pip install smf
```

---

## Usage & API Reference

1. **High-Speed Terminal Output (`sml.printf`)**  
Using an interface identical to `print()`, but executed in the Rust FFI layer:
```python
import sml

# Custom separators & terminators
sml.printf("A", "B", "C", sep=" | ", end="\n---\n")

# Unpacking payload besar tanpa I/O lag
large_payload = [f"Data_{i}" for i in range(100_000)]
sml.printf(*large_payload, sep=", ")

# Stream redirection ke file object
with open("system.log", "a") as f:
    sml.printf("System status: OK", file=f, flush=True)
```

2. **Rust FFI Type Handling**  
`smlog` handle Python data type conversions efficiently at the Rust level:
```python
class CustomObject:
    def __str__(self):
        return "<CustomObject String Representation>"

# Handles PyBytes natively (escaped)
bytes = b"Hello\nWorld\x00"
sml.printf("Raw Bytes:", bytes)

# Handles NoneType & Custom Objects via __str__ slot
sml.printf("None Type:", None)
sml.printf("Custom Class:", CustomObject())
```

3. **Isolated SQLite Debug Logging (sml.printd)**  
Save debug state and traceback to SQLite in OS temporary directory (`/tmp`):
```python
try:
    result = 10 / 0
except Exception as e:
    # Automatically saved in SQLite without polluting the terminal stdout
    sml.printd("Division failed", e, level="ERROR")
```

---


## Technical Architecture & PyO3 Integration

`smlog` designed as a high-performance C-Extension that bridges **Python Global Interpreter Lock (GIL)** with **Rust Native Concurrency/I/O Engine**.

```ddl
  +---------------------------------------------------------------------------+
  |                               Python Layer                                |
  |  sml.printf(*args, sep, end, file, flush)      sml.printd(*args, level)   |
  +-------------------------------------+-------------------------------------+
                                        | PyO3 FFI Boundary
  +-------------------------------------v-------------------------------------+
  |                          Rust Native Engine (sml)                         |
  |                                                                           |
  |         +--------------------+             +--------------------+         |
  |         | Fast Type Resolver |             | Traceback Extractor|         |
  |         | (PyBytes/PyStr)    |             | (PyErr/Exception)  |         |
  |         +---------+----------+             +---------+----------+         |
  |                   |                                  |                    |
  |                   v                                  v                    |
  |         +--------------------+             +--------------------+         |
  |         | Direct OS stdout / |             | SQLite Connection  |         |
  |         | BufWriter Engine   |             | Pool (WAL Mode)    |         |
  |         +---------+----------+             +---------+----------+         |
  +-------------------|----------------------------------|--------------------+
                      v                                  v
               System Terminal                 OS /tmp/smlog/log.db (0o700)
```

1. **PyO3 Type Ingestion & FFI Conversion**  
Crucial points in performance `smlog` is how Python data types are converted to Rust without excessive memory allocation overhead:
- **`PyBytes` Ingestion:** Caught using `obj.downcast::<PyBytes>()`. Byte streams are processed directly at the Rust buffer level and non-printable characters are escaped automatically.
- **`NoneType` Isolation:** Evaluated directly with C API preprocessing via `obj.is_none()`, avoiding Python attribute calls.
- **Custom Object Handling:** Call slot `__str__` on C-Struct Python via `obj.str()` only if the object is not a primitive type (string, int, float, bytes, bool).

2. **Lock & Thread Safety Design**  
- `sml.printf`: Minimize reading duration GIL (Global Interpreter Lock). Concatenated string formatting (string concatenation) performed in the Rust thread layer before being executed to standard output.
- `sml.printd`: Use **SQLite Write-Ahead Logging (WAL) Mode** which is stored in the OS's built-in temporary directory (`/tmp` or `%TEMP%`). Log writing is done in a thread-safe manner using an isolated connection pool to avoid database locked concerns when logs are sent in parallel/massively.

---

## SQLite Database Schema (DDL)

To ensure that the `sml.printd` query can execute debugging logs and large-capacity tracebacks without causing performance degradation, the following SQLite database schema is automatically applied during module initialization:

```sql
-- Database Location: OS Temporary Directory (e.g., /tmp/smlog/log.db)
-- journal mode = WAL (Write Concurrency)
-- synchronous = NORMAL (Balanced Durability)
-- temp_store = MEMORY (RAM Temp Storage)

CREATE TABLE IF NOT EXISTS system_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp REAL,
    level TEXT,
    label TEXT,
    payload TEXT,
    traceback TEXT,
    caller_info TEXT
);",
```

### SQLite Log Schema (sml.printd)

Log data is stored in the OS temporary database with the following schema:

| Field | Type | Description |
| :--- | :--- | :--- |
| timestamp | DATETIME | Time the log was created (ISO-8601 UTC) |
| level | TEXT | Log severity (DEBUG, INFO, ERROR, WARN) |
| label | TEXT | Taken from the first string |
| payload | TEXT | Argument fusion result string |
| traceback | TEXT | Captured Python exception stack trace (If there are) |
| caller_info | TEXT | Location of the script caller that caused the error |

---

## License

This tool is distributed under the [GPL License](LICENSE).



