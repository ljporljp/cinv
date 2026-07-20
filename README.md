# cinv

Chinese citizen identification number validator​ compliant with GB 11643‑1999​ standard.

校验中国居民身份证号码（18位）的合法性，严格遵循 GB 11643‑1999《公民身份号码》​ 国家标准。
--- 

[![Crates.io](https://img.shields.io/crates/v/cinv.svg)](https://crates.io/crates/cinv)
[![Downloads](https://img.shields.io/crates/d/cinv)](https://crates.io/crates/cinv)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![docs.rs](https://docs.rs/cinv/badge.svg)](https://docs.rs/cinv)
[![dependency status](https://deps.rs/repo/github/ljporljp/cinv/status.svg)](https://deps.rs/repo/github/ljporljp/cinv)
![MSRV](https://img.shields.io/badge/MSRV-1.80.0-blue)

## Features

✅ 18 位格式校验：长度、字符集、地址码首位

✅ 出生日期合法性校验：闰年/平年、各月天数（纯标准库，无 chrono 依赖

✅ MOD 11‑2 校验码：ISO 7064 MOD 11‑2 算法

✅ 地址码校验：内置民政部省/地/县三级行政区划代码

✅ 两种匹配模式：精确 6 位匹配 / 6→4→2 回溯兼容

✅ 支持自定义地址码：可传入外部 HashMap 覆盖内置数据

✅ 零外部依赖：仅 Rust 标准库，无 serde / chrono 等依赖

✅ 零 unwrap：所有错误路径安全处理，无 panic 风险

✅ 线程安全：内置地址码使用 LazyLock惰性初始化

## Quick Start

最简单的用法
``` rust

use cinv::is_valid;

// 合法身份证号
assert!(is_valid("11010519491231002X"));

// 非法身份证号
assert!(!is_valid("110105194912310021")); // 校验码错误
assert!(!is_valid("11010519491331002X")); // 月份 13
assert!(!is_valid(""));                   // 空字符串

```

使用自定义地址码
``` rust

use std::collections::HashMap;
use cinv::is_valid_with_map;

let mut codes = HashMap::new();
codes.insert("330106", "浙江省杭州市西湖区");
codes.insert("440104", "广东省广州市越秀区");
assert!(is_valid_with_map("330106199003071236", &codes));
assert!(!is_valid_with_map("999999199003071234", &codes));

```

选择地址码匹配模式
``` rust

use cinv::{is_valid_with_mode, AddressMatchMode};

// 精确模式：仅匹配 6 位县级代码
assert!(is_valid_with_mode("330106199003071236", AddressMatchMode::Exact6));

// 回溯模式：6 位不匹配时，回溯到 4 位地级码、2 位省级码
assert!(is_valid_with_mode("330109199003071235", AddressMatchMode::Fallback64));

```

同时使用自定义地址码和匹配模式
``` rust

use std::collections::HashMap;
use cinv::{is_valid_with_map_and_mode, AddressMatchMode};

let mut codes = HashMap::new();
codes.insert("330100", "浙江省杭州市"); 

// 精确模式：330109 不在表中，失败
assert!(!is_valid_with_map_and_mode(
    "330109199003071235",
    &codes,
    AddressMatchMode::Exact6,
)); 

// 回溯模式：330109 不在表中，但 330100 在表中，成功
assert!(is_valid_with_map_and_mode(
    "330109199003071235",
    &codes,
    AddressMatchMode::Fallback64,
));

```

## Data

更新地址码
当民政部发布最新数据后，录入data/address_codes.csv，运行项目中的生成脚本即可生成默认内置地址码：
``` bash

# 1. 下载最新的 address_codes.csv

# 2. 运行生成脚本

python scripts/generate_address_codes.py data/address_codes.csv src/address_codes.rs

```

然后重新编译即可
cargo build

## Project Structure

``` text
cinv/

├── Cargo.toml

├── README.md

├── LICENSE

├── src/

│   ├── lib.rs                 # 主逻辑（校验、测试）

│   └── address_codes.rs       # 自动生成的地址码数据

├── scripts/

│   └── generate_address_codes.py  # 地址码生成脚本

└── data/

    └── address_codes.csv               # 原始地址码数据源

```

``` bash

# 运行所有测试
cargo test  

# 生成测试身份证号
cargo test test_generate_valid_id
```

## Performance  

内置地址码使用 LazyLock静态初始化，首次调用后零开销
- 内置地址码使用 LazyLock静态初始化，首次调用后零开销
- HashMap查找时间复杂度 O(1)
- 单次校验仅需微秒级，适合高并发场景  

## License 

This project is licensed under the MIT License - see the LICENSE file for details.

## References 

- GB 11643‑1999：《公民身份号码》
- GB/T 2260：《中华人民共和国行政区划代码》
- ISO 7064 MOD 11‑2：校验码算法标准
 
## Made with ❤️ for the Rust community
