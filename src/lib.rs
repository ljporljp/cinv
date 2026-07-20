mod address_codes;

use address_codes::DEFAULT_ADDRESS_CODES;
use std::collections::HashMap;

/// ---------- 校验模式 ----------
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressMatchMode {
    /// 仅精确匹配 6 位地址码（最严格）
    Exact6,
    /// 6 位不匹配时，回溯到 4 位地级码，再回溯到 2 位省级码（兼容历史变动）
    Fallback64,
}

impl Default for AddressMatchMode {
    fn default() -> Self {
        AddressMatchMode::Fallback64
    }
}

/// ---------- 公开接口1：用内置地址码 + 默认模式（Fallback64）----------
pub fn is_valid(id: &str) -> bool {
    is_valid_with_mode(id, AddressMatchMode::default())
}

/// ---------- 公开接口2：用内置地址码 + 指定模式 ----------
pub fn is_valid_with_mode(
    id: &str,
    mode: AddressMatchMode,
) -> bool {
    is_valid_with_map_and_mode(id, &DEFAULT_ADDRESS_CODES, mode)
}

/// ---------- 公开接口3：用外部地址码 + 默认模式 ----------
pub fn is_valid_with_map(id: &str, codes: &HashMap<&str, &str>) -> bool {
    is_valid_with_map_and_mode(id, codes, AddressMatchMode::default())
}

/// ---------- 公开接口4：用外部地址码 + 指定模式（最灵活） ----------
pub fn is_valid_with_map_and_mode(
    id: &str,
    codes: &HashMap<&str, &str>,
    mode: AddressMatchMode,
) -> bool {
    let id = id.trim();

    // 1. 长度
    if id.len() != 18 {
        return false;
    }

    // 2. 前17位必须是数字
    let body = &id[..17];
    if !body.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // 3. 第18位
    let last = match id.chars().nth(17) {
        Some(c) => c.to_ascii_uppercase(),
        None => return false,
    };
    if !(last.is_ascii_digit() || last == 'X') {
        return false;
    }

    // 4. 地址码首位不能为0
    match id.chars().next() {
        Some('0') => return false,
        None => return false,
        _ => {}
    }

    // 5. 出生日期
    if !is_valid_date(&id[6..14]) {
        return false;
    }

    // 6. 校验码 MOD 11-2
    if !verify_checksum(body, last) {
        return false;
    }

    // 7. 地址码存在性（按模式选择策略）
    match mode {
        AddressMatchMode::Exact6 => is_address_exact6(id, codes),
        AddressMatchMode::Fallback64 => is_address_fallback64(id, codes),
    }
}

/// ---------- 日期校验（纯标准库） ----------
pub(crate) fn is_valid_date(s: &str) -> bool {
    if s.len() != 8 {
        return false;
    }

    let year = match s[0..4].parse::<u32>() {
        Ok(v) => v,
        Err(_) => return false,
    };
    let month = match s[4..6].parse::<u32>() {
        Ok(v) => v,
        Err(_) => return false,
    };
    let day = match s[6..8].parse::<u32>() {
        Ok(v) => v,
        Err(_) => return false,
    };

    if month < 1 || month > 12 {
        return false;
    }

    let days_in_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let max_day = if month == 2 && is_leap_year(year) {
        29
    } else {
        days_in_month[(month - 1) as usize]
    };

    day >= 1 && day <= max_day
}

/// ---------- 闰年判断 ----------
#[inline]
pub(crate) fn is_leap_year(y: u32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

/// ---------- MOD 11-2 校验码 ----------
pub(crate) const WEIGHTS: [u32; 17] =
    [7, 9, 10, 5, 8, 4, 2, 1, 6, 3, 7, 9, 10, 5, 8, 4, 2];

pub(crate) const CHECK_MAP: &[u8; 11] = b"10X98765432";

/// ---------- 校验码验证 ----------
pub(crate) fn verify_checksum(body: &str, last: char) -> bool {
    let mut sum = 0u32;

    for (i, c) in body.chars().enumerate() {
        let d = match c.to_digit(10) {
            Some(v) => v,
            None => return false,
        };
        sum += d * WEIGHTS[i];
    }

    let expected = CHECK_MAP[(sum % 11) as usize] as char;
    expected == last
}

/// ---------- 地址码校验：仅精确 6 位 ----------
/// 地址码进行 6→4→2 回溯（即精确 6 位不匹配时，回溯到 4 位地级/直辖市汇总码、再回溯到 2 位省级码），
/// 根本原因是你的配置文件（HashMap）的数据完整度与国标 GB 11643‑1999 中地址码的实际层级之间存在现实差距，主要出于以下三点考量：

/// 配置文件的裁剪与层级缺失
/// GB/T 2260的行政区划代码是六级嵌套的（省 2 位、地市 2 位、县区 2 位，拼成 6 位）。但在实际工程中：
/// 很多内置或自定义 JSON 只配置了常用的县级 6 位码，或者只配了省级/地级汇总码（如 110000北京市，330100杭州市）。
/// 如果一个身份证是合法的 330106（西湖区），但你的配置里只存了 330000（浙江省），不做回溯就会直接判为非法，导致大量真实老号码被拦截。

/// 历史行政区划变动的兼容
/// 身份证号一旦分配终身不变，但行政区划会合并、撤县设区、改名甚至撤销：
/// 某人早年发证用的是旧县码（如已撤销的 XXXXXX），该码在最新的民政部表里可能被移除。
/// 但该号码在法律上依然有效。回溯到上级 4 位或 2 位码（上级码相对稳定），可以在不丢失严格性的前提下，
/// 承认其所属的上一级行政区是合法的，避免误杀历史存量数据。

/// 国标语义上的合理性
/// GB 11643‑1999 规定前 6 位为县（市、旗、区）级代码，其结构是严格递进的：
/// 1‑2 位：省级（省、自治区、直辖市）
/// 3‑4 位：地级（市、州、盟）
/// 5‑6 位：县级（区、县、旗）
/// 当精确的 6 位码因配置不全查不到时，只要其上级行政隶属关系在配置中存在（如 33浙江、3301杭州），就可以认为地址码“结构合法”。
/// 这是一种降级兼容校验，常用于配置数据非全量加载的场景。
pub(crate) fn is_address_exact6(id: &str, codes: &HashMap<&str, &str>) -> bool {
    let code6 = match id.get(0..6) {
        Some(c) => c,
        None => return false,
    };
    codes.contains_key(code6)
}

/// ---------- 地址码校验：6 → 4 → 2 回溯 ----------
pub(crate) fn is_address_fallback64(id: &str, codes: &HashMap<&str, &str>) -> bool {
    let code6 = match id.get(0..6) {
        Some(c) => c,
        None => return false,
    };
    if codes.contains_key(code6) {
        return true;
    }

    let code4 = match id.get(0..4) {
        Some(c) => format!("{}00", c),
        None => return false,
    };
    if codes.contains_key(code4.as_str()) {
        return true;
    }

    let code2 = match id.get(0..2) {
        Some(c) => format!("{}0000", c),
        None => return false,
    };
    if codes.contains_key(code2.as_str()) {
        return true;
    }

    false
}

/// 生成合法的测试身份证号
/// 
/// # 参数
/// - `area_code`: 6位地址码
/// - `birth_date`: 8位出生日期（YYYYMMDD）
/// - `seq`: 3位顺序码
/// 
/// # 返回值
/// - `Some(String)`: 生成的18位合法身份证号
/// - `None`: 输入参数不合法（包含非数字字符等）
pub fn generate_test_id_with_address_code(area_code: &str, birth_date: &str, seq: &str) -> Option<String> {
    let base = format!("{}{}{}", area_code, birth_date, seq);
    
    // 检查长度
    if base.len() != 17 {
        return None;
    }

    let mut sum: u32 = 0;
    for (i, c) in base.chars().enumerate() {
        let digit = match c.to_digit(10) {
            Some(d) => d,
            None => return None,  // 包含非数字字符
        };
        sum += digit * WEIGHTS[i];
    }
    let check = CHECK_MAP[(sum % 11) as usize] as char;
    Some(format!("{}{}", base, check))
}
