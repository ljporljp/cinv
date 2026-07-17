#!/usr/bin/env python3
# ===== 关键修复：在脚本最开头强制设置编码环境 =====
# cmd /c "set PYTHONIOENCODING=utf-8 && python scripts/generate_address_codes.py data/area_code.csv src/address_codes.rs"
import sys
import locale

# 强制设置标准输出的编码为 UTF-8
if hasattr(sys.stdout, 'reconfigure'):
    sys.stdout.reconfigure(encoding='utf-8')
if hasattr(sys.stderr, 'reconfigure'):
    sys.stderr.reconfigure(encoding='utf-8')

# 设置环境变量，影响所有子进程
import os
os.environ['PYTHONIOENCODING'] = 'utf-8'

# 强制设置 locale
locale.setlocale(locale.LC_ALL, 'en_US.UTF-8')
# =================================================

import csv
from datetime import datetime

def pad_code(code: str) -> str:
    """将 2/4/6 位 code 补全为 6 位"""
    code = code.strip()
    if len(code) == 2:
        return code + "0000"
    elif len(code) == 4:
        return code + "00"
    elif len(code) == 6:
        return code
    else:
        return None

def generate_address_codes_file(csv_path: str) -> str:
    """生成完整的 Rust address_codes.rs 文件内容"""
    
    inserts = []
    seen_codes = set()
    errors = []
    
    try:
        # 以二进制模式读取，手动处理编码
        with open(csv_path, 'rb') as f:
            raw_bytes = f.read()
        
        # 检测并移除 BOM
        if raw_bytes[:3] == b'\xef\xbb\xbf':
            raw_bytes = raw_bytes[3:]
        
        # 解码为字符串
        content = raw_bytes.decode('utf-8')
        
        # 使用 StringIO 模拟文件对象
        import io
        string_io = io.StringIO(content)
        reader = csv.DictReader(string_io)
        
        required_fields = ['code', 'name', 'level']
        for field in required_fields:
            if field not in reader.fieldnames:
                return f"// 错误：CSV 缺少必要字段 '{field}'\n// 可用字段: {reader.fieldnames}\n"
        
        for row_num, row in enumerate(reader, start=2):
            code = row['code'].strip()
            name = row['name'].strip()
            level = row['level'].strip()
            
            if level not in ('1', '2', '3'):
                continue
            
            padded_code = pad_code(code)
            if padded_code is None:
                errors.append(f"警告：第 {row_num} 行 code '{code}' 长度非法，已跳过")
                continue
            
            if padded_code in seen_codes:
                continue
            seen_codes.add(padded_code)
            
            escaped_name = name.replace('\\', '\\\\').replace('"', '\\"')
            inserts.append((padded_code, escaped_name))
    
    except FileNotFoundError:
        return f"// 错误：找不到文件 '{csv_path}'\n"
    except Exception as e:
        import traceback
        return f"// 错误：读取 CSV 时发生异常: {e}\n// {traceback.format_exc()}\n"
    
    inserts.sort(key=lambda x: x[0])
    
    # 构建完整的 Rust 文件内容
    lines = []
    
    # 文件头注释
    lines.append("// ============================================================")
    lines.append("// 自动生成的地址码数据文件")
    lines.append(f"// 数据来源: {os.path.basename(csv_path)}")
    lines.append(f"// 生成时间: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    lines.append(f"// 条目数: {len(inserts)}")
    lines.append("// ============================================================")
    lines.append("")
    
    # use 声明
    lines.append("use std::collections::HashMap;")
    lines.append("use std::sync::LazyLock;")
    lines.append("")
    
    # static 定义
    lines.append("/// 静态内置地址码（省/地/县三级，6位）")
    lines.append("pub static DEFAULT_ADDRESS_CODES: LazyLock<HashMap<&'static str, &'static str>> =")
    lines.append("    LazyLock::new(|| {")
    lines.append("        let mut m = HashMap::with_capacity(3500);")
    lines.append("")
    
    # insert 语句
    for code, name in inserts:
        lines.append(f'        m.insert("{code}", "{name}");')
    
    lines.append("")
    lines.append("        m")
    lines.append("    });")
    
    # 如果有警告信息，追加到文件末尾
    if errors:
        lines.append("")
        lines.append("/*")
        lines.append(" * 生成过程中的警告信息：")
        for err in errors:
            lines.append(f" * {err}")
        lines.append(" */")
    
    return "\n".join(lines)

def write_utf8_file_absolute(filepath: str, content: str):
    """绝对确保以 UTF-8 编码写入文件"""
    # 将内容编码为 UTF-8 字节
    utf8_bytes = content.encode('utf-8')
    
    # 以二进制模式写入
    with open(filepath, 'wb') as f:
        f.write(utf8_bytes)
    
    # 验证写入结果
    with open(filepath, 'rb') as f:
        verification = f.read(10)
    
    # 检查是否为 UTF-16LE
    if verification[:2] == b'\xff\xfe':
        raise RuntimeError("文件被错误地写成了 UTF-16LE 编码！")
    
    # 检查是否为 UTF-8 with BOM
    if verification[:3] == b'\xef\xbb\xbf':
        print("警告：文件包含 UTF-8 BOM", file=sys.stderr)

def main():
    if len(sys.argv) < 2:
        print("用法:", file=sys.stderr)
        print(f"  {sys.argv[0]} <data.csv> [output.rs]", file=sys.stderr)
        print("", file=sys.stderr)
        print("示例:", file=sys.stderr)
        print(f"  {sys.argv[0]} data/area_code.csv src/address_codes.rs", file=sys.stderr)
        print(f"  {sys.argv[0]} data/area_code.csv > src/address_codes.rs", file=sys.stderr)
        sys.exit(1)
    
    csv_path = sys.argv[1]
    
    if not os.path.exists(csv_path):
        print(f"错误：文件不存在 '{csv_path}'", file=sys.stderr)
        sys.exit(1)
    
    rust_code = generate_address_codes_file(csv_path)
    
    if len(sys.argv) >= 3:
        output_path = sys.argv[2]
        try:
            write_utf8_file_absolute(output_path, rust_code)
            
            print(f"✓ 已生成: {output_path}", file=sys.stderr)
            print(f"  - 共 {rust_code.count('m.insert')} 条地址码", file=sys.stderr)
            
            # 统计各级数量
            level_counts = {"省级": 0, "地级": 0, "县级": 0}
            for line in rust_code.split('\n'):
                if 'm.insert("' in line:
                    code = line.split('"')[1]
                    if code.endswith("0000"):
                        level_counts["省级"] += 1
                    elif code.endswith("00"):
                        level_counts["地级"] += 1
                    else:
                        level_counts["县级"] += 1
            
            for level, count in level_counts.items():
                if count > 0:
                    print(f"  - {level}: {count}", file=sys.stderr)
                    
        except Exception as e:
            print(f"错误：写入文件失败: {e}", file=sys.stderr)
            sys.exit(1)
    else:
        # 使用二进制模式输出到 stdout
        sys.stdout.buffer.write(rust_code.encode('utf-8'))
        sys.stdout.buffer.write(b'\n')

if __name__ == "__main__":
    main()