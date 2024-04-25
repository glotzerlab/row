# JSON pointers

[JSON pointers](https://datatracker.ietf.org/doc/html/rfc6901) provide a general
way for you to access elements of a directory's JSON value.

For example, given the JSON document:
```
{
  "a": 1,
  "b": {
    "c": "str1",
    "d": 3.14
  },
  "e": [
    1,
    2,
    "str2"
  ]
}
```
you can access any element with a pointer:
| JSON pointer | value |
|--------------|-------|
| `"/a"` | `1` |
| `"/b"` | `{"c": "str1", "d": 3.14}` |
| `"/b/c"` | `"str1"` |
| `"/b/d"` | `3.14` |
| `"/b/e"` | `[1, 2, "str2"]` |
| `"/b/e/0"` | `1` |
| `"/b/e/1"` | `2` |
| `"/b/e/2"` | `"str2"` |

Read the [JSON pointer specification](https://datatracker.ietf.org/doc/html/rfc6901)
for more details.
