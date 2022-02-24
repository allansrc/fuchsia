<!-- WARNING: This file is machine generated by //src/tests/fidl/source_compatibility/gen, do not edit. -->

Note: This document covers API impact only. For more details, see the
[ABI compatibility page](/docs/development/languages/fidl/guides/compatibility/README.md)

# Change a union from strict to flexible

## Overview

-|[init](#init)|[step 1](#step-1)|[step 2](#step-2)|[step 3](#step-3)
---|---|---|---|---
fidl|[link](#fidl-init)||[link](#fidl-2)|
dart|[link](#dart-init)|[link](#dart-1)||[link](#dart-3)
go|[link](#go-init)|||[link](#go-3)
hlcpp|[link](#hlcpp-init)|[link](#hlcpp-1)||[link](#hlcpp-3)
llcpp|[link](#llcpp-init)|[link](#llcpp-1)||[link](#llcpp-3)
rust|[link](#rust-init)|[link](#rust-1)||[link](#rust-3)

## Initial State {#init}

### FIDL {#fidl-init}

```fidl
type JsonValue = strict union {
    1: int_value int32;
    2: string_value string:MAX;
};
```

### Dart {#dart-init}

```dart
void useUnion(fidllib.JsonValue value) {
  assert(value.$unknownData == null);
  switch (value.$tag) {
    case fidllib.JsonValueTag.intValue:
      print('int value: ${value.intValue}');
      break;
    case fidllib.JsonValueTag.stringValue:
      print('string value: ${value.stringValue}');
      break;
  }
}

```

### Go {#go-init}

```go
func useUnion(value lib.JsonValue) {
	switch value.Which() {
	case lib.JsonValueIntValue:
		fmt.Printf("int value: %d\n", value.IntValue)
	case lib.JsonValueStringValue:
		fmt.Printf("string value: %s\n", value.StringValue)
	default:
		fmt.Println("unknown tag")
	}
}

```

### HLCPP {#hlcpp-init}

```cpp
void use_union(fidl_test::JsonValue value) {
  switch (value.Which()) {
    case fidl_test::JsonValue::Tag::kIntValue:
      printf("int value: %d\n", value.int_value());
      break;
    case fidl_test::JsonValue::Tag::kStringValue:
      printf("string value: %s\n", value.string_value().c_str());
      break;
    case fidl_test::JsonValue::Tag::Invalid:
      printf("<uninitialized union>\n");
      break;
  }
}

```

### LLCPP {#llcpp-init}

```cpp
void use_union(fidl_test::wire::JsonValue* value) {
  switch (value->which()) {
    case fidl_test::wire::JsonValue::Tag::kIntValue:
      printf("int value: %d\n", value->int_value());
      break;
    case fidl_test::wire::JsonValue::Tag::kStringValue:
      printf("string value: %s\n", value->string_value().data());
      break;
  }
}
```

### Rust {#rust-init}

```rust
fn use_union(value: &fidl_lib::JsonValue) {
    match value {
        fidl_lib::JsonValue::IntValue(n) => println!("int value: {}", n),
        fidl_lib::JsonValue::StringValue(s) => println!("string: {}", s),
    };
}
```

## Update Source Code {#step-1}

### Dart {#dart-1}

- Add a default case to any switch statements on the union to handle new unknown variants

```diff
  void useUnion(fidllib.JsonValue value) {
-   assert(value.$unknownData == null);
    switch (value.$tag) {
      case fidllib.JsonValueTag.intValue:
        print('int value: ${value.intValue}');
        break;
      case fidllib.JsonValueTag.stringValue:
        print('string value: ${value.stringValue}');
        break;
+     default:
+       // Note: unknown variants will fail to decode until the union is marked flexible
+       print('unknown variant: ${value.$unknownData}');
+       break;
    }
  }
  

```

### HLCPP {#hlcpp-1}

- Add a default case to any switch statements on the union to handle new unknown variants

```diff
  void use_union(fidl_test::JsonValue value) {
    switch (value.Which()) {
      case fidl_test::JsonValue::Tag::kIntValue:
        printf("int value: %d\n", value.int_value());
        break;
      case fidl_test::JsonValue::Tag::kStringValue:
        printf("string value: %s\n", value.string_value().c_str());
        break;
      case fidl_test::JsonValue::Tag::Invalid:
        printf("<uninitialized union>\n");
        break;
+     default:
+       printf("<unknown variant>\n");
    }
  }
  

```

### LLCPP {#llcpp-1}

- Add a default case to any switch statements on the union to handle new unknown variants

```diff
  void use_union(fidl_test::wire::JsonValue* value) {
    switch (value->which()) {
      case fidl_test::wire::JsonValue::Tag::kIntValue:
        printf("int value: %d\n", value->int_value());
        break;
      case fidl_test::wire::JsonValue::Tag::kStringValue:
        printf("string value: %s\n", value->string_value().data());
        break;
+     default:
+       printf("<unknown variant>\n");
    }
  }

```

### Rust {#rust-1}

- Add `[allow(unreachable_patterns)]` and an underscore arm to match statements on the union

```diff
  fn use_union(value: &fidl_lib::JsonValue) {
+     #[allow(unreachable_patterns)]
      match value {
          fidl_lib::JsonValue::IntValue(n) => println!("int value: {}", n),
          fidl_lib::JsonValue::StringValue(s) => println!("string: {}", s),
+         _ => {}
      };
  }

```

## Update FIDL Library {#step-2}

- Change the union from `strict` to `flexible`

```diff
- type JsonValue = strict union {
+ type JsonValue = flexible union {
      1: int_value int32;
      2: string_value string:MAX;
  };

```

## Update Source Code {#step-3}

### Dart {#dart-3}

- Replace the default case with the unknown tag.

```diff
  void useUnion(fidllib.JsonValue value) {
    switch (value.$tag) {
      case fidllib.JsonValueTag.intValue:
        print('int value: ${value.intValue}');
        break;
      case fidllib.JsonValueTag.stringValue:
        print('string value: ${value.stringValue}');
        break;
-     default:
-       // Note: unknown variants will fail to decode until the union is marked flexible
+     case fidllib.JsonValueTag.$unknown:
        print('unknown variant: ${value.$unknownData}');
        break;
    }
  }
  

```

### Go {#go-3}

- You may now use any flexible union specific APIs

```diff
  func useUnion(value lib.JsonValue) {
  	switch value.Which() {
  	case lib.JsonValueIntValue:
  		fmt.Printf("int value: %d\n", value.IntValue)
  	case lib.JsonValueStringValue:
  		fmt.Printf("string value: %s\n", value.StringValue)
+ 	case lib.JsonValue_unknownData:
+ 		fmt.Printf("unknown data: %+v\n", value.GetUnknownData())
  	default:
  		fmt.Println("unknown tag")
  	}
  }
  

```

### HLCPP {#hlcpp-3}

- Replace the default case with the `kUnknown` tag.
- You may now use any flexible union specific APIs

```diff
  void use_union(fidl_test::JsonValue value) {
    switch (value.Which()) {
      case fidl_test::JsonValue::Tag::kIntValue:
        printf("int value: %d\n", value.int_value());
        break;
      case fidl_test::JsonValue::Tag::kStringValue:
        printf("string value: %s\n", value.string_value().c_str());
        break;
      case fidl_test::JsonValue::Tag::Invalid:
        printf("<uninitialized union>\n");
        break;
-     default:
-       printf("<unknown variant>\n");
+     case fidl_test::JsonValue::Tag::kUnknown:
+       printf("<%lu unknown bytes>\n", value.UnknownBytes()->size());
+       break;
    }
  }
  

```

### LLCPP {#llcpp-3}

- Replace the default case with the `kUnknown` tag.
- You may now use any flexible union specific APIs

```diff
  void use_union(fidl_test::wire::JsonValue* value) {
    switch (value->which()) {
      case fidl_test::wire::JsonValue::Tag::kIntValue:
        printf("int value: %d\n", value->int_value());
        break;
      case fidl_test::wire::JsonValue::Tag::kStringValue:
        printf("string value: %s\n", value->string_value().data());
        break;
-     default:
-       printf("<unknown variant>\n");
+     case fidl_test::wire::JsonValue::Tag::kUnknown:
+       printf("<unknown data>\n");
+       break;
    }
  }

```

### Rust {#rust-3}

- Remove the attribute and replace the underscore arm with the generated macro to match against unknown variants

```diff
  fn use_union(value: &fidl_lib::JsonValue) {
-     #[allow(unreachable_patterns)]
      match value {
          fidl_lib::JsonValue::IntValue(n) => println!("int value: {}", n),
          fidl_lib::JsonValue::StringValue(s) => println!("string: {}", s),
-         _ => {}
+         fidl_lib::JsonValueUnknown!() => println!("<unknown union>"),
      };
  }

```
