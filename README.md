cinv

Chinese citizen identification number validator​ compliant with GB 11643‑1999​ standard.

校验中国居民身份证号码（18位）的合法性，严格遵循 GB 11643‑1999《公民身份号码》​ 国家标准。

Features

✅ 18 位格式校验：长度、字符集、地址码首位

✅ 出生日期合法性校验：闰年/平年、各月天数（纯标准库，无 chrono 依赖）

✅ MOD 11‑2 校验码：ISO 7064 MOD 11‑2 算法

✅ 地址码校验：内置民政部省/地/县三级行政区划代码

✅ 两种匹配模式：精确 6 位匹配 / 6→4→2 回溯兼容

✅ 支持自定义地址码：可传入外部 HashMap 覆盖内置数据

✅ 零外部依赖：仅 Rust 标准库，无 serde / chrono 等依赖

✅ 零 unwrap：所有错误路径安全处理，无 panic 风险

✅ 线程安全：内置地址码使用 LazyLock惰性初始化

Installation

在 Cargo.toml中添加：

toml
[dependencies]
cinv = "0.1.0"
Quick Start
最简单的用法
rust
use cinv::is_valid;

// 合法身份证号
assert!(is_valid("11010519491231002X"));

// 非法身份证号
assert!(!is_valid("110105194912310021")); // 校验码错误
assert!(!is_valid("11010519491331002X")); // 月份 13
assert!(!is_valid(""));                   // 空字符串
使用自定义地址码
rust
use std::collections::HashMap;
use cinv::is_valid_with_map;

let mut codes = HashMap::new();
codes.insert("330106", "浙江省杭州市西湖区");
codes.insert("440104", "广东省广州市越秀区");

assert!(is_valid_with_map("330106199003071236", &codes));
assert!(!is_valid_with_map("999999199003071234", &codes));
选择地址码匹配模式
rust
use cinv::{is_valid_with_mode, AddressMatchMode};

// 精确模式：仅匹配 6 位县级代码
assert!(is_valid_with_mode("330106199003071236", AddressMatchMode::Exact6));

// 回溯模式：6 位不匹配时，回溯到 4 位地级码、2 位省级码
assert!(is_valid_with_mode("330109199003071235", AddressMatchMode::Fallback64));
同时使用自定义地址码和匹配模式
rust
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
API Reference
Public Functions

Function

	

Description




is_valid(id: &str) -> bool

	

使用内置地址码 + 默认模式（Fallback64）校验




is_valid_with_mode(id: &str, mode: AddressMatchMode) -> bool

	

使用内置地址码 + 指定模式校验




is_valid_with_map(id: &str, codes: &HashMap<&str, &str>) -> bool

	

使用外部地址码 + 默认模式校验




is_valid_with_map_and_mode(id: &str, codes: &HashMap<&str, &str>, mode: AddressMatchMode) -> bool

	

使用外部地址码 + 指定模式校验




generate_test_id(area_code: &str, birth_date: &str, seq: &str) -> Option<String>

	

生成合法的测试身份证号

AddressMatchMode

Variant

	

Description




Exact6

	

仅精确匹配 6 位地址码（最严格）




Fallback64

	

6 位不匹配时，回溯到 4 位地级码，再回溯到 2 位省级码（兼容历史变动）

Validation Rules

校验按照 GB 11643‑1999 标准依次进行：

长度：必须为 18 位

字符集：前 17 位必须为数字，第 18 位为数字或大写 X

地址码首位：不能为 0

出生日期：8 位 YYYYMMDD 格式，需为合法公历日期（含闰年判断）

校验码：ISO 7064 MOD 11‑2 算法

地址码存在性：前 6 位必须在地址码表中（根据选择的匹配模式）

Built-in Address Codes

库内置了民政部截至 2025 年末​ 的省/地/县三级行政区划代码：

Level

	

Count

	

Example




省级（2 位 → 补 0000）

	

34

	

110000北京市




地级（4 位 → 补 00）

	

~380

	

330100杭州市




县级（6 位）

	

~2800

	

330106西湖区

数据来源：国家地名信息库
及民政部年度发布。

更新地址码

当民政部发布最新数据后，运行项目中的生成脚本即可更新：

bash
# 1. 下载最新的 address_codes.csv
# 2. 运行生成脚本
python scripts/generate_address_codes.py data/address_codes.csv src/address_codes.rs

# 3. 重新编译
cargo build
Project Structure
纯文本
cn_id_validator/
├── Cargo.toml
├── README.md
├── LICENSE
├── src/
│   ├── lib.rs                 # 主逻辑（校验、测试）
│   └── address_codes.rs       # 自动生成的地址码数据
├── scripts/
│   └── generate_address_codes.py  # 地址码生成脚本
└── data/
    └── address_codes.csv               # 原始地址码数据源
Testing
bash
# 运行所有测试
cargo test

# 运行特定测试（显示输出）
cargo test valid -- --nocapture

# 生成测试身份证号
cargo test test_generate_valid_id -- --nocapture
Performance

内置地址码使用 LazyLock静态初始化，首次调用后零开销

HashMap查找时间复杂度 O(1)

单次校验仅需微秒级，适合高并发场景

License

This project is licensed under the MIT License - see the LICENSE
file for details.

References

GB 11643‑1999：《公民身份号码》

GB/T 2260：《中华人民共和国行政区划代码》

ISO 7064 MOD 11‑2：校验码算法标准

Changelog
0.1.0 (2026-07-17)

Initial release

18 位身份证号码校验

内置省/地/县三级地址码

支持精确/回溯两种匹配模式

支持自定义地址码

零外部依赖，零 unwrap

Made with ❤️ for the Rust community