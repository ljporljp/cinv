use cinv::*;
use std::collections::HashMap;

#[test]
fn test_valid_default_mode() {
    assert!(is_valid("11010519491231002X"));
}

#[test]
fn test_exact6_mode() {
    // 330106 西湖区 在内置表中
    assert!(is_valid_with_mode("330106199003071236", AddressMatchMode::Exact6));

    // 330109 萧山区 不在内置表中（示例表没配），Exact6 模式下应失败
    assert!(!is_valid_with_mode("330109199003071239", AddressMatchMode::Exact6));
}

#[test]
fn test_fallback64_mode() {
    // 330109 萧山区 不在内置表中，但 330100 杭州市在表中，Fallback64 下应成功
    assert!(is_valid_with_mode("330109199003071235", AddressMatchMode::Fallback64));
}

#[test]
fn test_with_external_map() {
    let mut custom = HashMap::new();
    custom.insert("330106", "浙江省杭州市西湖区");

    // 默认模式（Fallback64）
    assert!(is_valid_with_map("330106199003071236", &custom));

    // 精确模式
    assert!(is_valid_with_map_and_mode(
        "330106199003071236",
        &custom,
        AddressMatchMode::Exact6,
    ));

    // 330109 不在自定义表中，精确模式应失败
    assert!(!is_valid_with_map_and_mode(
        "330109199003071239",
        &custom,
        AddressMatchMode::Exact6,
    ));

    // 330109 不在自定义表中，但 330100 也不在，回溯也会失败
    assert!(!is_valid_with_map_and_mode(
        "330109199003071239",
        &custom,
        AddressMatchMode::Fallback64,
    ));
}

#[test]
fn test_invalid_examples() {
    assert!(!is_valid("110105194912310021")); // 校验码错误
    assert!(!is_valid("11010519491331002X")); // 月份13
    assert!(!is_valid("11010519490229002X")); // 1949年2月29
    assert!(!is_valid("01010519491231002X")); // 地址码首位0
    assert!(!is_valid(""));                    // 空字符串
}

#[test]
fn test_generate_valid_id() {
    let id = generate_test_id_with_address_code("330109", "19900307", "123");
    assert_eq!(id, Some("330109199003071235".into()));
    assert!(is_valid(id.as_ref().unwrap()));

    let id2 = generate_test_id_with_address_code("330109", "19900307", "001");
    assert_eq!(id2, Some("330109199003070013".into()));
    assert!(is_valid(id2.as_ref().unwrap()));

    // 生成正确的测试身份证号
    let id = generate_test_id_with_address_code("330109", "19900307", "123");
    assert!(id.is_some());
    
    let id_str = id.unwrap();  // 这里 unwrap 是安全的，因为上面已经检查过了
    dbg!(&id_str);
    
    // 验证生成的号码可以通过校验
    assert!(is_valid(&id_str));
}

#[test]
fn test_generate_id_with_invalid_input() {
    // 包含非数字字符
    let id = generate_test_id_with_address_code("33010A", "19900307", "123");
    assert!(id.is_none());
    
    // 长度不足
    let id = generate_test_id_with_address_code("3301", "19900307", "123");
    assert!(id.is_none());
}

#[test]
fn test_fallback64_mode_with_generated_id() {
    // 使用生成的正确身份证号
    let id = generate_test_id_with_address_code("330109", "19900307", "123").unwrap();
    assert!(is_valid_with_mode(&id, AddressMatchMode::Fallback64));
}

#[test]
fn test_exact6_mode_with_generated_id() {
    // 330106 西湖区 在内置表中
    let id = generate_test_id_with_address_code("330106", "19900307", "123").unwrap();
    assert!(is_valid_with_mode(&id, AddressMatchMode::Exact6));

    // 330115 不在内置表中，Exact6 模式下应失败
    let id2 = generate_test_id_with_address_code("330115", "19900307", "123").unwrap();
    assert!(!is_valid_with_mode(&id2, AddressMatchMode::Exact6));
}
