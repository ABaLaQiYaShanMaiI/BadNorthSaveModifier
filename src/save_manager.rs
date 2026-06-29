use anyhow::{anyhow, Result};
use log::{info, warn, debug};
use std::fs;
use std::path::Path;
use serde_json::Value;
use paste::paste;
use bad_north_save_converter::parser::parse;
use bad_north_save_converter::json::{JsonDecoder, JsonEncoder};
use bad_north_save_converter::serializer::serialize_checked;

pub const GRAIL_UPGRADE_CODE: &str = "Hero_Upgrade_Grail";

pub const BOMB_UPGRADE_CODE: &str = "Hero_Upgrade_Bomb";
pub const MINE_UPGRADE_CODE: &str = "Hero_Upgrade_Mine";
pub const PHILOSOPHERS_STONE_UPGRADE_CODE: &str = "Hero_Upgrade_PhilosophersStone";
pub const SIZE_UPGRADE_CODE: &str = "Hero_Upgrade_Size";
pub const WARHAMMER_UPGRADE_CODE: &str = "Hero_Upgrade_Warhammer";
pub const CORNUCOPIA_UPGRADE_CODE: &str = "Hero_Upgrade_Cornucopia";
pub const WAR_HORN_UPGRADE_CODE: &str = "Hero_Upgrade_WarHorn";

// ============ 快捷方法宏 (生成类似物品的 get/set/increment/decrement 方法) ============
macro_rules! item_count_shortcuts {
    ($name:ident, $code:expr) => {
        paste! {
            pub fn [<get_inventory_ $name _count>](json_value: &Value) -> i32 {
                Self::get_inventory_item_count(json_value, $code)
            }
            pub fn [<get_hero_ $name _count>](json_value: &Value) -> i32 {
                Self::get_hero_item_count(json_value, $code)
            }
            pub fn [<get_total_ $name _count>](json_value: &Value) -> i32 {
                Self::get_total_item_count(json_value, $code)
            }
            pub fn [<set_ $name _count>](json_value: &mut Value, target: i32) -> Result<()> {
                Self::set_item_count(json_value, $code, target)
            }
            pub fn [<increment_ $name _count>](json_value: &mut Value) -> Result<i32> {
                let current_total = Self::get_total_inventory_count(json_value);
                if current_total >= 20 {
                    return Err(anyhow!("❌背包已满（{}/20），无法继续添加装备", current_total));
                }
                Self::increment_item_count(json_value, $code)
            }
            pub fn [<decrement_ $name _count>](json_value: &mut Value) -> Result<i32> {
                Self::decrement_item_count(json_value, $code)
            }
        }
    };
    (@dead_code $name:ident, $code:expr) => {
        paste! {
            #[allow(dead_code)]
            pub fn [<get_inventory_ $name _count>](json_value: &Value) -> i32 {
                Self::get_inventory_item_count(json_value, $code)
            }
            #[allow(dead_code)]
            pub fn [<get_hero_ $name _count>](json_value: &Value) -> i32 {
                Self::get_hero_item_count(json_value, $code)
            }
            #[allow(dead_code)]
            pub fn [<get_total_ $name _count>](json_value: &Value) -> i32 {
                Self::get_total_item_count(json_value, $code)
            }
            #[allow(dead_code)]
            pub fn [<set_ $name _count>](json_value: &mut Value, target: i32) -> Result<()> {
                Self::set_item_count(json_value, $code, target)
            }
            #[allow(dead_code)]
            pub fn [<increment_ $name _count>](json_value: &mut Value) -> Result<i32> {
                let current_total = Self::get_total_inventory_count(json_value);
                if current_total >= 20 {
                    return Err(anyhow!("❌背包已满（{}/20），无法继续添加装备", current_total));
                }
                Self::increment_item_count(json_value, $code)
            }
            #[allow(dead_code)]
            pub fn [<decrement_ $name _count>](json_value: &mut Value) -> Result<i32> {
                Self::decrement_item_count(json_value, $code)
            }
        }
    };
}

pub struct SaveManager;

impl SaveManager {
    // ============ 代码结构说明 ============
    // 本文件包含 10 个功能模块，总代码行数 ~2100 行
    // 
    // 模块 1-4: 基础操作（文件I/O、内部工具、英雄升级、属性）
    // 模块 5-7: 数据查询（查询、货币、圣杯）
    // 模块 8-10: 背包管理（通用、快捷方法、摘要）
    // 
    // 优化机会：模块 9 的 56 个快捷方法可用宏生成
    // 预计可将 150+ 行代码简化为 ~10 行宏调用
    // =========================================

    // ============ File I/O & Serialization (使用库直接内存操作) ============
    pub fn load_save(save_path: &Path) -> Result<Value> {
        if !save_path.exists() {
            return Err(anyhow!("存档文件不存在：{}", save_path.display()));
        }

        info!("正在加载存档：{}", save_path.display());

        let bytes = fs::read(save_path)
            .map_err(|e| anyhow!("读取存档文件失败：{}", e))?;

        let rec = parse(&bytes)
            .map_err(|e| anyhow!("解析二进制存档失败：{}", e))?;

        let json_value = JsonEncoder::new(rec).encode();

        info!("存档已加载（内存转换，无需外部EXE）");
        Ok(json_value)
    }

    /// Kept for backward compatibility with callers still passing editor_exe (ignored)
    #[allow(dead_code)]
    pub fn load_save_with_exe(save_path: &Path, _editor_exe: &Path) -> Result<Value> {
        Self::load_save(save_path)
    }

    pub fn export_json(json_value: &Value, export_path: &Path) -> Result<()> {
        let json_content = serde_json::to_string_pretty(json_value)?;
        fs::write(export_path, json_content)?;
        info!("已导出 JSON 到：{}", export_path.display());
        Ok(())
    }

    pub fn save_save(save_path: &Path, json_value: &Value) -> Result<()> {
        info!("正在序列化并保存存档：{}", save_path.display());

        let rec = JsonDecoder::decode(json_value)
            .map_err(|e| anyhow!("JSON 解码失败：{}", e))?;

        let output_bytes = serialize_checked(&rec)
            .map_err(|e| anyhow!("二进制序列化失败：{}", e))?;

        // Create backup
        let backup_path = save_path.with_extension("backup");
        if save_path.exists() {
            fs::copy(save_path, &backup_path)?;
            info!("已创建备份：{:?}", backup_path);
        }

        fs::write(save_path, output_bytes)?;
        info!("存档已保存（内存转换，无需外部EXE）");
        Ok(())
    }

    /// Kept for backward compatibility with callers still passing editor_exe (ignored)
    #[allow(dead_code)]
    pub fn save_save_with_exe(save_path: &Path, json_value: &Value, _editor_exe: &Path) -> Result<()> {
        Self::save_save(save_path, json_value)
    }

    fn normalize_upgrade_type(upgrade_type: &str) -> String {
        match upgrade_type {

            "classUpgrade" | "itemUpgrade" | "traitUpgrade" | "skillUpgrade" | "upgrade" => {
                upgrade_type.to_string()
            }

            "classUpgrades" => "classUpgrade".to_string(),
            "itemUpgrades"  => "itemUpgrade".to_string(),
            "traitUpgrades" => "traitUpgrade".to_string(),
            "skillUpgrades" => "skillUpgrade".to_string(),
            "upgrades"      => "upgrade".to_string(),

            "class_upgrade" => "classUpgrade".to_string(),
            "item_upgrade"  => "itemUpgrade".to_string(),
            "trait_upgrade" => "traitUpgrade".to_string(),
            "skill_upgrade" => "skillUpgrade".to_string(),

            "class_upgrades" => "classUpgrade".to_string(),
            "item_upgrades"  => "itemUpgrade".to_string(),
            "trait_upgrades" => "traitUpgrade".to_string(),
            "skill_upgrades" => "skillUpgrade".to_string(),

            other => other.to_string(),
        }
    }


    fn create_virtual_upgrade_record(
        json_value: &mut Value,
        upgrade_type: &str,
    ) -> Result<String> {

        let max_id = json_value["records"]
            .as_object()
            .ok_or_else(|| anyhow!("Records not found"))?
            .keys()
            .filter_map(|k| k.parse::<i64>().ok())
            .max()
            .unwrap_or(0);

        let new_id = (max_id + 1) as i32;
        let new_id_str = new_id.to_string();

        let (class_type_id, class_type_name) = Self::find_serializable_hero_upgrade_class_type(json_value)
            .unwrap_or((0, "Voxels.TowerDefense.SerializableHeroUpgrade".to_string()));

        let new_upgrade_record = serde_json::json!({
            "type": "Class",
            "class_type_id": class_type_id,
            "class_type_name": class_type_name,
            "members": [
                {
                    "name": "name",
                    "type": "String",
                    "value": ""
                },
                {
                    "name": "level",
                    "type": "Int32",
                    "value": 0
                }
            ]
        });

        json_value["records"][&new_id_str] = new_upgrade_record;

        let metadata = Self::get_upgrade_record_metadata_template(json_value);
        if let Some(meta_obj) = json_value.get_mut("record_metadata").and_then(|m| m.as_object_mut()) {
            meta_obj.insert(new_id_str.clone(), metadata);
            info!("Registered virtual record {} in record_metadata", new_id);
        }

        if let Some(order) = json_value.get_mut("record_order").and_then(|v| v.as_array_mut()) {
            order.push(Value::Number(new_id.into()));
        }

        info!("Created virtual {} record {}", upgrade_type, new_id);
        Ok(new_id_str)
    }



    fn get_upgrade_record_metadata_template(json_value: &Value) -> Value {
        let records = json_value["records"].as_object();
        let meta_obj = json_value["record_metadata"].as_object();
        let found = records.zip(meta_obj).and_then(|(recs, metas)| {
            recs.iter()
                .find(|(id, rec)| {
                    rec["class_type_name"].as_str()
                        .map(|n| n.contains("SerializableHeroUpgrade"))
                        .unwrap_or(false)
                        && metas.contains_key(*id)
                })
                .and_then(|(id, _)| metas.get(id).cloned())
        });
        found.unwrap_or_else(|| serde_json::json!({






            "original_byte_length": 50,
            "original_record_type": 5,
            "start_offset": 0,
            "end_offset": 50
        }))
    }


    pub fn modify_hero_upgrade(
        json_value: &mut Value,
        hero_record_id: &str,
        upgrade_type: &str,
        new_name: &str,
        new_level: i32,
    ) -> Result<()> {

        let normalized_type = Self::normalize_upgrade_type(upgrade_type);

        let upgrade_ref_id: String = {

            let ref_id: Option<String> = {
                let hero_record = json_value["records"][hero_record_id].as_object()
                    .ok_or_else(|| anyhow!("Hero record {} not found", hero_record_id))?;
                let members = hero_record["members"].as_array()
                    .ok_or_else(|| anyhow!("Members not found in hero record {}", hero_record_id))?;

                let mut r: Option<String> = None;
                for member in members.iter() {
                    let mname = member["name"].as_str().unwrap_or("");
                    if mname == normalized_type {
                        if let Some(id) = member["id"].as_i64() {
                            debug!("Found {} (field={}) reference id={}", normalized_type, mname, id);
                            r = Some(id.to_string());
                            break;
                        }
                    }
                }
                r
            };

            match ref_id {
                Some(id) => id,
                None => {

                    info!("Upgrade field {} is null, creating virtual record", normalized_type);
                    let new_id = Self::create_virtual_upgrade_record(json_value, &normalized_type)?;

                    if let Some(hero_rec) = json_value["records"][hero_record_id].as_object_mut() {
                        if let Some(members_arr) = hero_rec.get_mut("members").and_then(|m| m.as_array_mut()) {
                            for member in members_arr.iter_mut() {
                                if member["name"].as_str() == Some(normalized_type.as_str()) {
                                    if let Some(obj) = member.as_object_mut() {
                                        obj.insert("type".to_string(), Value::String("Reference".to_string()));
                                        obj.insert("id".to_string(), Value::Number(new_id.parse::<i64>()?.into()));
                                        obj.remove("value");
                                    }
                                    info!("Updated hero {} field {} to reference {}",
                                          hero_record_id, normalized_type, new_id);
                                    break;
                                }
                            }
                        }
                    }

                    new_id
                }
            }
        };

        let (old_name, old_level, name_string_ref_id): (String, i32, Option<String>) = {
            let upgrade_record = json_value["records"][&upgrade_ref_id].as_object()
                .ok_or_else(|| anyhow!("Upgrade record {} not found", upgrade_ref_id))?;
            let members = upgrade_record["members"].as_array()
                .ok_or_else(|| anyhow!("Members not found in upgrade record {}", upgrade_ref_id))?;

            let mut cur_name = String::new();
            let mut cur_level = 0i32;
            let mut str_ref: Option<String> = None;

            for member in members.iter() {
                match member["name"].as_str() {
                    Some("name") => {
                        if let Some(ref_id_int) = member["id"].as_i64() {

                            let sref = ref_id_int.to_string();
                            let name_record = &json_value["records"][&sref];
                            if let Some(s) = name_record.as_str() {
                                cur_name = s.to_string();
                            } else if let Some(obj) = name_record.as_object() {
                                if let Some(s) = obj.get("value").and_then(|v| v.as_str()) {
                                    cur_name = s.to_string();
                                }
                            }
                            str_ref = Some(sref);
                        } else if let Some(s) = member["value"].as_str() {

                            cur_name = s.to_string();
                        }
                    }
                    Some("level") => {
                        if let Some(l) = member["value"].as_i64() {
                            cur_level = l as i32;
                        }
                    }
                    _ => {}
                }
            }
            (cur_name, cur_level, str_ref)
        };


        if normalized_type == "itemUpgrade" || normalized_type == "traitUpgrade" {
            if old_name.contains("Cornucopia") || new_name.contains("Cornucopia") {
                warn!(
                    "⚠️操作涉及雅贝那（Cornucopia）：旧值「{}」→ 新值「{}」。\
                    雅贝那效果由游戏引擎缓存，修改后建议重启游戏以刷新效果。",
                    old_name, new_name
                );
            }
        }

        let old_bytes = old_name.len();
        let new_bytes = new_name.len();
        info!(
            "正在修改 {} (hero={}): name=\"{}\" ({}字节, L{}) →\"{}\" ({}字节, L{})",
            normalized_type, hero_record_id,
            old_name, old_bytes, old_level,
            new_name, new_bytes, new_level
        );
        if !old_name.is_empty() && old_bytes != new_bytes {
            warn!(
                "⚠️字符串长度变化：原始值「{}」{} 字节) →新值「{}」{} 字节)，差值{} 字节。\
                若保存时出现 'Record size mismatch' 错误，原因在于游戏二进制格式要求各记录大小不变，\
                但字段「{}」的字符串长度发生了变化（原 {} 字节，现 {} 字节）。",
                old_name, old_bytes, new_name, new_bytes,
                new_bytes as i32 - old_bytes as i32,
                normalized_type, old_bytes, new_bytes
            );
        }

        if let Some(ref str_id) = name_string_ref_id {

            if json_value["records"][str_id].is_object() {
                if let Some(name_record) = json_value["records"][str_id].as_object_mut() {
                    if let Some(members_arr) = name_record.get_mut("members").and_then(|m| m.as_array_mut()) {
                        for member in members_arr.iter_mut() {
                            if member["name"].as_str() == Some("value") {
                                member["value"] = Value::String(new_name.to_string());
                                debug!("Updated string record {} (via members) to \"{}\"", str_id, new_name);
                                break;
                            }
                        }
                    } else {
                        name_record.insert("value".to_string(), Value::String(new_name.to_string()));
                        debug!("Updated string record {} (top-level value) to \"{}\"", str_id, new_name);
                    }
                }
            } else {

                json_value["records"][str_id] = Value::String(new_name.to_string());
                debug!("Replaced plain string record {} with \"{}\"", str_id, new_name);
            }
        } else {

            if let Some(upgrade_record) = json_value["records"][&upgrade_ref_id].as_object_mut() {
                if let Some(members_arr) = upgrade_record["members"].as_array_mut() {
                    for member in members_arr.iter_mut() {
                        if member["name"].as_str() == Some("name") {
                            member["value"] = Value::String(new_name.to_string());
                            debug!("Updated upgrade record {} name directly to \"{}\"", upgrade_ref_id, new_name);
                            break;
                        }
                    }
                }
            }
        }

        if let Some(upgrade_record) = json_value["records"][&upgrade_ref_id].as_object_mut() {
            if let Some(members_arr) = upgrade_record["members"].as_array_mut() {
                let mut found_level = false;
                for member in members_arr.iter_mut() {
                    if member["name"].as_str() == Some("level") {
                        member["value"] = Value::Number(new_level.into());
                        debug!("Updated upgrade record {} level to {}", upgrade_ref_id, new_level);
                        found_level = true;
                        break;
                    }
                }
                if !found_level {
                    members_arr.push(serde_json::json!({
                        "name": "level",
                        "type": "Int32",
                        "value": new_level
                    }));
                    debug!("Added level field to upgrade record {} with value {}", upgrade_ref_id, new_level);
                }
            }
        }

        info!(
            "Modified {} for hero {} →name=\"{}\" level={}",
            normalized_type, hero_record_id, new_name, new_level
        );

        if normalized_type == "classUpgrade" {
            if let Err(e) = Self::clear_skill_upgrade(json_value, hero_record_id) {
                warn!("Failed to clear skill upgrade for hero {}: {}", hero_record_id, e);
            }
        }

        Ok(())
    }

    pub fn clear_skill_upgrade(
        json_value: &mut Value,
        hero_record_id: &str,
    ) -> Result<()> {
        let hero_record = json_value["records"][hero_record_id].as_object_mut()
            .ok_or_else(|| anyhow!("Failed to find hero record {} for clearing skill upgrade", hero_record_id))?;

        let members = hero_record["members"].as_array_mut()
            .ok_or_else(|| anyhow!("Failed to find members array in hero record {} for clearing skill upgrade", hero_record_id))?;

        for member in members.iter_mut() {
            if member["name"].as_str() == Some("skillUpgrade") {
                member["type"] = Value::String("Null".to_string());
                member["value"] = Value::Null;

                if let Some(obj) = member.as_object_mut() {
                    obj.remove("id");
                }

                info!("Cleared skill upgrade for hero {}", hero_record_id);
                return Ok(());
            }
        }

        Ok(())
    }


    pub fn clear_hero_upgrade(
        json_value: &mut Value,
        hero_record_id: &str,
        upgrade_type: &str,
    ) -> Result<()> {
        let normalized_type = Self::normalize_upgrade_type(upgrade_type);

        let hero_record = json_value["records"][hero_record_id].as_object_mut()
            .ok_or_else(|| anyhow!("Hero record {} not found", hero_record_id))?;

        let members = hero_record["members"].as_array_mut()
            .ok_or_else(|| anyhow!("Members not found in hero record {}", hero_record_id))?;

        for member in members.iter_mut() {
            if member["name"].as_str() == Some(normalized_type.as_str()) {
                member["type"] = Value::String("Null".to_string());
                member["value"] = Value::Null;

                if let Some(obj) = member.as_object_mut() {
                    obj.remove("id");
                }

                info!("Cleared {} for hero {}", normalized_type, hero_record_id);
                return Ok(());
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_class_skill_upgrade(class_code: &str) -> Option<&'static str> {
        match class_code {
            "Hero_Class_Infantry" => Some("Hero_Upgrade_Plunge"),
            "Hero_Class_Archers"  => Some("Hero_Upgrade_ArcheryFocus"),
            "Hero_Class_Pikemen"  => Some("Hero_Upgrade_PikeCharge"),
            _ => None,
        }
    }


    #[allow(dead_code)]
    pub fn sync_class_skill(
        json_value: &mut Value,
        hero_record_id: &str,
        class_code: &str,
    ) -> Result<()> {

        let skill_upgrade_code = Self::get_class_skill_upgrade(class_code)
            .ok_or_else(|| anyhow!("Unknown class code: {}", class_code))?;

        let hero_record = json_value["records"][hero_record_id].as_object_mut()
            .ok_or_else(|| anyhow!("Hero record {} not found", hero_record_id))?;

        let members = hero_record["members"].as_array_mut()
            .ok_or_else(|| anyhow!("Members not found in hero record {}", hero_record_id))?;

        let mut skill_upgrade_id: Option<i64> = None;

        for member in members.iter() {
            if member["name"].as_str() == Some("skillUpgrade") {
                if let Some(id) = member.get("id").and_then(|v| v.as_i64()) {
                    skill_upgrade_id = Some(id);
                    break;
                }
            }
        }


        if skill_upgrade_id.is_none() {
            info!(
                "⚠️英雄 {} 没有有效的skillUpgrade 字段，跳过兵种技能同步",
                hero_record_id
            );
            return Ok(());
        }

        let skill_record_id = skill_upgrade_id
            .ok_or_else(|| anyhow!("Failed to get skillUpgrade ID"))?
            .to_string();

        if let Some(record) = json_value["records"][&skill_record_id].as_object_mut() {
            if let Some(members_arr) = record.get_mut("members").and_then(|m| m.as_array_mut()) {
                for member in members_arr.iter_mut() {
                    if member["name"].as_str() == Some("name") {

                        if let Some(_name_ref_id) = member.get("id").and_then(|v| v.as_i64()) {

                            member["value"] = Value::String(skill_upgrade_code.to_string());
                        } else {

                            member["value"] = Value::String(skill_upgrade_code.to_string());
                        }
                        debug!("Updated skillUpgrade name to \"{}\"", skill_upgrade_code);
                        break;
                    }
                }
            }
        }

        info!(
            "✔兵种技能已同步: {} →{} (skillUpgrade id={})",
            class_code, skill_upgrade_code, skill_record_id
        );

        Ok(())
    }

    #[allow(dead_code)]
    pub fn modify_hero_coins(
        json_value: &mut Value,
        hero_record_id: &str,
        new_coins: i32,
    ) -> Result<()> {
        let hero_record = json_value["records"][hero_record_id].as_object_mut()
            .ok_or_else(|| anyhow!("Hero record {} not found", hero_record_id))?;

        let members = hero_record["members"].as_array_mut()
            .ok_or_else(|| anyhow!("Members array not found in hero record"))?;

        for member in members.iter_mut() {
            if member["name"].as_str() == Some("_coins") {
                member["value"] = Value::Number(new_coins.into());
                info!("Modified coins for hero {} to {}", hero_record_id, new_coins);
                return Ok(());
            }
        }

        Err(anyhow!("Coins field not found for hero {}", hero_record_id))
    }


    #[allow(dead_code)]
    pub fn modify_hero_helmet(
        json_value: &mut Value,
        hero_record_id: &str,
        new_hue: f32,
    ) -> Result<()> {
        let hero_record = json_value["records"][hero_record_id].as_object_mut()
            .ok_or_else(|| anyhow!("Hero record {} not found", hero_record_id))?;

        let members = hero_record["members"].as_array_mut()
            .ok_or_else(|| anyhow!("Members array not found in hero record"))?;

        for member in members.iter_mut() {
            if member["name"].as_str() == Some("hue") {
                member["value"] = Value::Number(
                    serde_json::Number::from_f64(new_hue as f64)
                        .ok_or_else(|| anyhow!("Invalid hue value"))?
                );
                info!("Modified helmet hue for hero {} to {}", hero_record_id, new_hue);
                return Ok(());
            }
        }

        Err(anyhow!("Hue field not found for hero {}", hero_record_id))
    }

    #[allow(dead_code)]
    pub fn modify_hero_crown(
        json_value: &mut Value,
        hero_record_id: &str,
        has_crown: bool,
    ) -> Result<()> {
        let hero_record = json_value["records"][hero_record_id].as_object_mut()
            .ok_or_else(|| anyhow!("Hero record {} not found", hero_record_id))?;

        let members = hero_record["members"].as_array_mut()
            .ok_or_else(|| anyhow!("Members array not found in hero record"))?;

        let default_crown_style = "Heroes2_Spriteshop_Crown_0".to_string();

        let mut has_crown_found = false;
        let mut crown_style_found = false;

        for member in members.iter_mut() {
            if member["name"].as_str() == Some("hasCrown") {
                member["value"] = Value::Bool(has_crown);
                has_crown_found = true;
            } else if member["name"].as_str() == Some("crownStyle") {
                if has_crown {
                    member["value"] = Value::String(default_crown_style.clone());
                } else {
                    member["value"] = Value::Null;
                }
                crown_style_found = true;
            }
        }

        if !has_crown_found {
            let has_crown_member = serde_json::json!({
                "name": "hasCrown",
                "type": "Boolean",
                "value": has_crown
            });
            members.push(has_crown_member);
        }

        if !crown_style_found {
            let crown_style_value = if has_crown {
                Value::String(default_crown_style)
            } else {
                Value::Null
            };
            let crown_style_member = serde_json::json!({
                "name": "crownStyle",
                "type": "String",
                "value": crown_style_value
            });
            members.push(crown_style_member);
        }

        info!(
            "Modified crown for hero {} to has_crown={}",
            hero_record_id, has_crown
        );

        Ok(())
    }

    pub fn get_recruited_heroes(json_value: &Value) -> Result<Vec<HeroDetails>> {
        let records = json_value["records"].as_object()
            .ok_or_else(|| anyhow!("Records not found in JSON"))?;

        let mut heroes = Vec::new();
        let empty_vec = vec![];

        for (record_id, record) in records.iter() {
            let class_type_name = record["class_type_name"].as_str().unwrap_or("");

            if class_type_name.contains("HeroDefinition") {
                let members = record["members"].as_array().unwrap_or(&empty_vec);

                let mut hero_id: Option<i32> = None;
                let mut recruited = false;

                for member in members {
                    match member["name"].as_str() {
                        Some("id") => {
                            if let Some(id) = member["value"].as_i64() {
                                hero_id = Some(id as i32);
                            }
                        }
                        Some("recruited") => {
                            recruited = member["value"].as_bool().unwrap_or(false);
                        }
                        _ => {}
                    }
                }

                if recruited {
                    if let Some(mut hero_detail) = Self::get_hero_details(json_value, record_id).ok() {
                        hero_detail.key = record_id.clone();
                        heroes.push(hero_detail);
                        debug!("Found recruited hero: record_id={}, hero_id={:?}", record_id, hero_id);
                    }
                }
            }
        }

        info!("Found {} recruited heroes", heroes.len());
        Ok(heroes)
    }

    pub fn get_hero_details(json_value: &Value, hero_record_id: &str) -> Result<HeroDetails> {
        let record = json_value["records"][hero_record_id].as_object()
            .ok_or_else(|| anyhow!("Hero record not found"))?;

        let members = record["members"].as_array()
            .ok_or_else(|| anyhow!("Members not found"))?;

        info!("英雄 {} 的完整members 结构：", hero_record_id);
        for (i, member) in members.iter().enumerate() {
            info!("  [{}] {}", i, member);
        }

        let mut details = HeroDetails::default();
        details.key = hero_record_id.to_string();

        for member in members {
            match member["name"].as_str() {
                Some("id") => {
                    if let Some(id) = member["value"].as_i64() {
                        details.id = id as i32;
                    }
                }
                Some("_coins") => {
                    if let Some(coins) = member["value"].as_i64() {
                        details.coins = coins as i32;
                    }
                }
                Some("hue") => {
                    if let Some(hue) = member["value"].as_f64() {
                        details.hue = hue as f32;
                    }
                }
                Some("hasCrown") => {
                    details.has_crown = member["value"].as_bool().unwrap_or(false);
                }
                Some("crownStyle") => {
                    if let Some(style) = member["value"].as_str() {
                        details.crown_style = Some(style.to_string());
                    }
                }
                Some("soldiersLost") => {
                    if let Some(v) = member["value"].as_i64() {
                        details.soldiers_lost = v as i32;
                    }
                }
                Some("maxSoldiers") => {
                    if let Some(v) = member["value"].as_i64() {
                        details.max_soldiers = v as i32;
                    }
                }
                Some("classUpgrade") => {
                    if let Some(ref_id) = member["id"].as_i64() {
                        details.class_upgrade_ref = Some(ref_id.to_string());
                    }
                }
                Some("itemUpgrade") => {
                    if let Some(ref_id) = member["id"].as_i64() {
                        details.item_upgrade_ref = Some(ref_id.to_string());
                    }
                }
                Some("traitUpgrade") => {
                    if let Some(ref_id) = member["id"].as_i64() {
                        details.trait_upgrade_ref = Some(ref_id.to_string());
                    }
                }
                Some("skillUpgrade") => {
                    if let Some(ref_id) = member["id"].as_i64() {
                        details.skill_upgrade_ref = Some(ref_id.to_string());
                    }
                }
                Some("upgrade") => {
                    if let Some(ref_id) = member["id"].as_i64() {
                        details.upgrade_ref = Some(ref_id.to_string());
                    }
                }
                _ => {}
            }
        }

        details.class_info = Self::get_upgrade_info(json_value, &details.class_upgrade_ref, "classUpgrade")?;
        details.item_info = Self::get_upgrade_info(json_value, &details.item_upgrade_ref, "itemUpgrade")?;
        details.trait_info = Self::get_upgrade_info(json_value, &details.trait_upgrade_ref, "traitUpgrade")?;
        details.skill_info = Self::get_upgrade_info(json_value, &details.skill_upgrade_ref, "skillUpgrade")?;
        details.upgrade_info = Self::get_upgrade_info(json_value, &details.upgrade_ref, "upgrade")?;

        Ok(details)
    }

    fn get_upgrade_info(json_value: &Value, ref_id: &Option<String>, field_name: &str) -> Result<Option<UpgradeInfo>> {
        let ref_id = match ref_id {
            Some(id) => id,
            None => return Ok(None),
        };

        let record = match json_value["records"][ref_id].as_object() {
            Some(r) => r,
            None => return Ok(None),
        };

        let members = match record["members"].as_array() {
            Some(m) => m,
            None => return Ok(None),
        };

        let mut upgrade_info = UpgradeInfo::default();
        upgrade_info.record_id = ref_id.clone();
        upgrade_info.field_name = field_name.to_string();

        for member in members {
            match member["name"].as_str() {
                Some("name") => {

                    if let Some(name_ref_id) = member["id"].as_i64() {

                        let name_record = &json_value["records"][&name_ref_id.to_string()];
                        if let Some(name_str) = name_record.as_str() {
                            upgrade_info.name = name_str.to_string();
                        } else if let Some(name_obj) = name_record.as_object() {

                            if let Some(name_str) = name_obj.get("value").and_then(|v| v.as_str()) {
                                upgrade_info.name = name_str.to_string();
                            }
                        }
                    } else if let Some(name_str) = member["value"].as_str() {

                        upgrade_info.name = name_str.to_string();
                    }
                }
                Some("level") => {
                    if let Some(level) = member["value"].as_i64() {
                        upgrade_info.level = level as i32;
                    }
                }
                _ => {}
            }
        }

        Ok(Some(upgrade_info))
    }

    pub fn get_coin_bank(json_value: &Value) -> Option<i32> {
        let records = json_value["records"].as_object()?;
        for (_record_id, record) in records.iter() {
            if let Some(members) = record["members"].as_array() {
                for member in members {
                    if member["name"].as_str() == Some("coinBank") {
                        if let Some(coins) = member["value"].as_i64() {
                            return Some(coins as i32);
                        }
                    }
                }
            }
        }
        None
    }

    pub fn modify_coin_bank(json_value: &mut Value, new_coins: i32) -> Result<()> {
        let records = json_value["records"].as_object_mut()
            .ok_or_else(|| anyhow!("Records not found in JSON"))?;
        for (_record_id, record) in records.iter_mut() {
            if let Some(members) = record["members"].as_array_mut() {
                for member in members.iter_mut() {
                    if member["name"].as_str() == Some("coinBank") {
                        member["value"] = Value::Number(new_coins.into());
                        info!("Modified coinBank to {}", new_coins);
                        return Ok(());
                    }
                }
            }
        }
        Err(anyhow!("coinBank field not found in save"))
    }

    fn read_upgrade_record_name(json_value: &Value, record_id: i64) -> String {
        let upgrade_record = &json_value["records"][&record_id.to_string()];
        let members = match upgrade_record["members"].as_array() {
            Some(m) => m,
            None => return String::new(),
        };
        for member in members {
            if member["name"].as_str() != Some("name") {
                continue;
            }
            if let Some(name_ref_id) = member["id"].as_i64() {
                let name_record = &json_value["records"][&name_ref_id.to_string()];
                if let Some(s) = name_record.as_str() {
                    return s.to_string();
                } else if let Some(obj) = name_record.as_object() {
                    return obj.get("value")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                }
            } else if let Some(s) = member["value"].as_str() {
                return s.to_string();
            }
        }
        String::new()
    }




    fn find_inventory_refs(json_value: &Value) -> Result<(Option<i64>, i64)> {
        let records = json_value["records"].as_object()
            .ok_or_else(|| anyhow!("records object not found"))?;

        for (_id, record) in records.iter() {
            let ctn = record["class_type_name"].as_str().unwrap_or("");
            if !ctn.contains("CampaignSave") {
                continue;
            }
            let members = match record["members"].as_array() {
                Some(m) => m,
                None => continue,
            };
            for member in members {
                if member["name"].as_str() != Some("inventory") {
                    continue;
                }
                let container_id = member["id"].as_i64()
                    .ok_or_else(|| anyhow!("inventory member has no id reference"))?;

                let container_record = &json_value["records"][&container_id.to_string()];

                if let Some(list_members) = container_record["members"].as_array() {
                    for lm in list_members {
                        if lm["name"].as_str() == Some("_items") {
                            if let Some(array_id) = lm["id"].as_i64() {
                                return Ok((Some(container_id), array_id));
                            }
                        }
                    }
                }

                if container_record["type"].as_str() == Some("BinaryArray") {
                    return Ok((None, container_id));
                }

                return Err(anyhow!(
                    "inventory reference {} is neither a List<T> nor a direct BinaryArray",
                    container_id
                ));
            }
        }
        Err(anyhow!("CampaignSave record with an inventory field not found"))
    }

    fn get_list_size(json_value: &Value, list_id: i64) -> i32 {
        let list_record = &json_value["records"][&list_id.to_string()];
        if let Some(members) = list_record["members"].as_array() {
            for member in members {
                if member["name"].as_str() == Some("_size") {
                    return member["value"].as_i64().unwrap_or(0) as i32;
                }
            }
        }
        0
    }

    fn set_list_size(json_value: &mut Value, list_id: i64, new_size: i32) -> Result<()> {
        let list_record = json_value["records"][&list_id.to_string()].as_object_mut()
            .ok_or_else(|| anyhow!("List record {} not found", list_id))?;
        let members = list_record["members"].as_array_mut()
            .ok_or_else(|| anyhow!("members not found in List record {}", list_id))?;
        for member in members.iter_mut() {
            if member["name"].as_str() == Some("_size") {
                member["value"] = Value::Number(new_size.into());
                return Ok(());
            }
        }
        Err(anyhow!("_size member not found in List record {}", list_id))
    }

    fn find_max_record_id(json_value: &Value) -> i64 {
        json_value["records"].as_object()
            .map(|m| {
                m.keys()
                    .filter_map(|k| k.parse::<i64>().ok())
                    .max()
                    .unwrap_or(0)
            })
            .unwrap_or(0)
    }

    fn find_serializable_hero_upgrade_class_type(json_value: &Value) -> Option<(u64, String)> {
        let records = json_value["records"].as_object()?;
        for (_id, record) in records.iter() {
            let ctn = record["class_type_name"].as_str().unwrap_or("");
            if ctn.contains("SerializableHeroUpgrade") {
                if let Some(type_id) = record["class_type_id"].as_u64() {
                    return Some((type_id, ctn.to_string()));
                }
            }
        }
        None
    }

    pub fn get_inventory_grail_count(json_value: &Value) -> i32 {
        let (list_id_opt, array_id) = match Self::find_inventory_refs(json_value) {
            Ok(refs) => refs,
            Err(_) => return 0,
        };

        let size_limit = list_id_opt.map(|lid| Self::get_list_size(json_value, lid));

        let values = match json_value["records"][&array_id.to_string()]["values"].as_array() {
            Some(v) => v,
            None => return 0,
        };

        let mut count = 0i32;
        let mut logical_idx = 0i32;

        for entry in values {
            if let Some(limit) = size_limit {
                if logical_idx >= limit {
                    break;
                }
            }
            match entry["type"].as_str() {
                Some("Reference") => {
                    if let Some(ref_id) = entry["id"].as_i64() {
                        if Self::read_upgrade_record_name(json_value, ref_id) == GRAIL_UPGRADE_CODE {
                            count += 1;
                        }
                    }
                    logical_idx += 1;
                }
                Some("NullMultiple") => {
                    let c = entry["count"].as_i64().unwrap_or(1) as i32;
                    logical_idx += c;
                }
                Some("Null") => {
                    logical_idx += 1;
                }
                _ => {
                    logical_idx += 1;
                }
            }
        }
        count
    }

    #[allow(dead_code)]
    pub fn get_total_grail_count(json_value: &Value) -> i32 {
        Self::get_hero_grail_count(json_value) + Self::get_inventory_grail_count(json_value)
    }

    #[allow(dead_code)]
    pub fn get_hero_grail_count(json_value: &Value) -> i32 {
        let records = match json_value["records"].as_object() {
            Some(r) => r,
            None => return 0,
        };

        let empty_vec = vec![];
        let mut count = 0;

        for (_record_id, record) in records.iter() {
            let class_type_name = record["class_type_name"].as_str().unwrap_or("");
            if !class_type_name.contains("HeroDefinition") {
                continue;
            }

            let members = record["members"].as_array().unwrap_or(&empty_vec);
            for member in members {
                if member["name"].as_str() == Some("upgrade") {
                    if let Some(ref_id) = member["id"].as_i64() {
                        if Self::read_upgrade_record_name(json_value, ref_id) == GRAIL_UPGRADE_CODE {
                            count += 1;
                        }
                    }
                    break;
                }
            }
        }
        count
    }

    #[allow(dead_code)]
    pub fn get_grail_count(json_value: &Value) -> i32 {
        Self::get_total_grail_count(json_value)
    }


    fn get_item_initial_level(_upgrade_code: &str) -> i32 {
        0
    }




    fn get_item_required_fields(_upgrade_code: &str) -> Vec<(&'static str, Value)> {
        vec![]
    }










    pub fn add_grail_to_inventory(json_value: &mut Value) -> Result<()> {
        let (list_id_opt, array_id) = Self::find_inventory_refs(json_value)?;

        let current_size = list_id_opt.map(|lid| Self::get_list_size(json_value, lid)).unwrap_or(0);

        let slot_index: usize;
        let null_multiple_count: i32;
        {
            let values = json_value["records"][&array_id.to_string()]["values"]
                .as_array()
                .ok_or_else(|| anyhow!("BinaryArray {} has no values array", array_id))?;

            let mut logical_idx = 0i32;
            let mut found: Option<(usize, i32)> = None;
            for (i, entry) in values.iter().enumerate() {
                match entry["type"].as_str() {
                    Some("Reference") => {
                        logical_idx += 1;
                    }
                    Some("NullMultiple") => {
                        if logical_idx >= current_size {
                            let c = entry["count"].as_i64().unwrap_or(1) as i32;
                            found = Some((i, c));
                            break;
                        }
                        let c = entry["count"].as_i64().unwrap_or(1) as i32;
                        logical_idx += c;
                    }
                    Some("Null") => {
                        if logical_idx >= current_size {
                            found = Some((i, 1));
                            break;
                        }
                        logical_idx += 1;
                    }
                    _ => {
                        logical_idx += 1;
                    }
                }
            }
            let (idx, nc) = found.ok_or_else(|| anyhow!("背包已满，没有可用的空余槽位"))?;
            slot_index = idx;
            null_multiple_count = nc;
        }

        let (class_type_id, class_type_name) = Self::find_serializable_hero_upgrade_class_type(json_value)
            .ok_or_else(|| anyhow!("找不到SerializableHeroUpgrade 类型信息"))?;

        let max_id = Self::find_max_record_id(json_value);
        let upgrade_id = max_id + 1;
        let upgrade_id_str = upgrade_id.to_string();

        let initial_level = Self::get_item_initial_level(GRAIL_UPGRADE_CODE);



        let extra_fields = Self::get_item_required_fields(GRAIL_UPGRADE_CODE);
        let mut members = vec![
            serde_json::json!({"name": "name",  "type": "String", "value": GRAIL_UPGRADE_CODE}),
            serde_json::json!({"name": "level", "type": "Int32",  "value": initial_level}),
        ];
        for (_, field_value) in extra_fields {
            members.push(field_value);
        }
        let upgrade_record = serde_json::json!({
            "type": "Class",
            "class_type_id": class_type_id,
            "class_type_name": class_type_name,
            "members": members
        });
        json_value["records"][&upgrade_id_str] = upgrade_record;

        let metadata = Self::get_upgrade_record_metadata_template(json_value);
        if let Some(meta_obj) = json_value.get_mut("record_metadata").and_then(|m| m.as_object_mut()) {
            meta_obj.insert(upgrade_id_str.clone(), metadata);
        }

        if let Some(order) = json_value.get_mut("record_order").and_then(|v| v.as_array_mut()) {
            let upgrade_id_i32 = i32::try_from(upgrade_id)
                .map_err(|_| anyhow!("record ID {} exceeds i32 range", upgrade_id))?;
            order.push(Value::Number(upgrade_id_i32.into()));
        }




        let new_entry = serde_json::json!({"type": "Reference", "id": upgrade_id});
        if let Some(values) = json_value["records"][&array_id.to_string()]["values"].as_array_mut() {
            if null_multiple_count > 1 {






                values[slot_index] = new_entry;


                let residual = serde_json::json!({"type": "NullMultiple", "count": null_multiple_count - 1});
                values.insert(slot_index + 1, residual);
            } else {

                values[slot_index] = new_entry;
            }
        }

        if let Some(lid) = list_id_opt {
            Self::set_list_size(json_value, lid, current_size + 1)?;
        }

        info!(
            "Added grail to inventory: upgrade_id={}, slot_index={}, null_multiple_count={}",
            upgrade_id, slot_index, null_multiple_count
        );
        if null_multiple_count <= 255 {
            warn!(
                "⚠️背包圣杯已写入JSON，但存储时可能失败：\
                BinaryArray 槽位原来是NullMultiple(count={})（ 字节），\
                替换为Reference（ 字节）会导致记录字节长度变化。\
                保存时若出现 'Record size mismatch' 错误，属于预期内的格式限制。",
                null_multiple_count
            );
        }
        Ok(())
    }




    pub fn remove_grail_from_inventory(json_value: &mut Value) -> Result<()> {
        let (list_id_opt, array_id) = Self::find_inventory_refs(json_value)?;
        let current_size = list_id_opt.map(|lid| Self::get_list_size(json_value, lid)).unwrap_or(0);

        let grail_json_index: usize;
        {
            let values = json_value["records"][&array_id.to_string()]["values"]
                .as_array()
                .ok_or_else(|| anyhow!("BinaryArray {} has no values array", array_id))?;

            let mut logical_idx = 0i32;
            let mut found: Option<usize> = None;
            for (i, entry) in values.iter().enumerate() {
                if logical_idx >= current_size {
                    break;
                }
                match entry["type"].as_str() {
                    Some("Reference") => {
                        if let Some(ref_id) = entry["id"].as_i64() {
                            if Self::read_upgrade_record_name(json_value, ref_id) == GRAIL_UPGRADE_CODE {
                                found = Some(i);
                                break;
                            }
                        }
                        logical_idx += 1;
                    }
                    Some("NullMultiple") => {
                        let c = entry["count"].as_i64().unwrap_or(1) as i32;
                        logical_idx += c;
                    }
                    Some("Null") => {
                        logical_idx += 1;
                    }
                    _ => {
                        logical_idx += 1;
                    }
                }
            }
            grail_json_index = found.ok_or_else(|| anyhow!("背包中没有圣杯可移除"))?;
        }

        let last_item_json_index: usize;
        {
            let values = json_value["records"][&array_id.to_string()]["values"]
                .as_array()
                .ok_or_else(|| anyhow!("BinaryArray {} has no values array", array_id))?;

            let mut logical_idx = 0i32;
            let target_logical = current_size - 1;
            let mut found: Option<usize> = None;
            for (i, entry) in values.iter().enumerate() {
                if logical_idx > target_logical {
                    break;
                }
                match entry["type"].as_str() {
                    Some("Reference") => {
                        if logical_idx == target_logical {
                            found = Some(i);
                            break;
                        }
                        logical_idx += 1;
                    }
                    Some("NullMultiple") => {
                        let c = entry["count"].as_i64().unwrap_or(1) as i32;
                        if logical_idx + c > target_logical {

                            break;
                        }
                        logical_idx += c;
                    }
                    Some("Null") => {
                        if logical_idx == target_logical {
                            found = Some(i);
                            break;
                        }
                        logical_idx += 1;
                    }
                    _ => {
                        logical_idx += 1;
                    }
                }
            }
            last_item_json_index = found.ok_or_else(|| anyhow!("找不到背包最后一个有效槽位"))?;
        }


        if let Some(values) = json_value["records"][&array_id.to_string()]["values"].as_array_mut() {
            if grail_json_index != last_item_json_index {
                values.swap(grail_json_index, last_item_json_index);
            }



            values[last_item_json_index] = serde_json::json!({"type": "NullMultiple", "count": 256});
        }

        if let Some(lid) = list_id_opt {
            Self::set_list_size(json_value, lid, current_size - 1)?;
        }

        info!("Removed grail from inventory, new size={}", current_size - 1);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn set_inventory_grail_count(json_value: &mut Value, target: i32) -> Result<()> {
        if target < 0 {
            return Err(anyhow!("圣杯数量不能为负数"));
        }
        let current = Self::get_inventory_grail_count(json_value);
        if target > current {
            for _ in 0..(target - current) {
                Self::add_grail_to_inventory(json_value)?;
            }
        } else if target < current {
            for _ in 0..(current - target) {
                Self::remove_grail_from_inventory(json_value)?;
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn add_grail(json_value: &mut Value) -> Result<()> {

        let target: Option<(String, i32)> = {
            let heroes = Self::get_recruited_heroes(json_value)?;
            heroes.into_iter().find_map(|hero| {
                match &hero.upgrade_info {
                    Some(info) if !info.name.is_empty() && info.name != GRAIL_UPGRADE_CODE => {
                        Some((hero.key.clone(), info.level))
                    }
                    _ => None,
                }
            })
        };

        if let Some((hero_key, level)) = target {
            Self::modify_hero_upgrade(json_value, &hero_key, "upgrade", GRAIL_UPGRADE_CODE, level)?;
            info!("Added grail to hero {}", hero_key);
            Ok(())
        } else {
            Err(anyhow!("未找到可用的英雄装备槽来添加圣杯（旧逻辑）"))
        }
    }

    #[allow(dead_code)]
    pub fn remove_grail(json_value: &mut Value) -> Result<()> {

        let target_key: Option<String> = {
            let heroes = Self::get_recruited_heroes(json_value)?;
            heroes.into_iter().find_map(|hero| {
                match &hero.upgrade_info {
                    Some(info) if info.name == GRAIL_UPGRADE_CODE => Some(hero.key.clone()),
                    _ => None,
                }
            })
        };

        if let Some(hero_key) = target_key {

            let hero_record = json_value["records"][&hero_key].as_object_mut()
                .ok_or_else(|| anyhow!("Hero record {} not found", hero_key))?;
            let members = hero_record["members"].as_array_mut()
                .ok_or_else(|| anyhow!("Members not found in hero record {}", hero_key))?;
            for member in members.iter_mut() {
                if member["name"].as_str() == Some("upgrade") {
                    member["type"] = Value::String("Null".to_string());
                    member["value"] = Value::Null;
                    if let Some(obj) = member.as_object_mut() {
                        obj.remove("id");
                    }
                    info!("Removed grail from hero {}", hero_key);
                    return Ok(());
                }
            }
            Err(anyhow!("Hero {} has no upgrade member to remove", hero_key))
        } else {
            Err(anyhow!("未找到携带圣杯的英雄（旧逻辑）"))
        }
    }

    #[allow(dead_code)]
    pub fn increment_grail_count(json_value: &mut Value) -> Result<i32> {
        Self::add_grail_to_inventory(json_value)?;
        Ok(Self::get_grail_count(json_value))
    }

    #[allow(dead_code)]
    pub fn decrement_grail_count(json_value: &mut Value) -> Result<i32> {
        Self::remove_grail_from_inventory(json_value)?;
        Ok(Self::get_grail_count(json_value))
    }

    #[allow(dead_code)]
    pub fn set_grail_count(json_value: &mut Value, target: i32) -> Result<()> {
        if target < 0 {
            return Err(anyhow!("圣杯数量不能为负数"));
        }
        let current = Self::get_grail_count(json_value);
        if target > current {
            for _ in 0..(target - current) {
                Self::increment_grail_count(json_value)?;
            }
        } else if target < current {
            for _ in 0..(current - target) {
                Self::decrement_grail_count(json_value)?;
            }
        }
        Ok(())
    }




    pub fn get_hero_item_count(json_value: &Value, upgrade_code: &str) -> i32 {
        let records = match json_value["records"].as_object() {
            Some(r) => r,
            None => return 0,
        };
        let empty_vec = vec![];
        let mut count = 0;
        for (_record_id, record) in records.iter() {
            let class_type_name = record["class_type_name"].as_str().unwrap_or("");
            if !class_type_name.contains("HeroDefinition") {
                continue;
            }
            let members = record["members"].as_array().unwrap_or(&empty_vec);
            for member in members {
                if member["name"].as_str() == Some("upgrade") {
                    if let Some(ref_id) = member["id"].as_i64() {
                        if Self::read_upgrade_record_name(json_value, ref_id) == upgrade_code {
                            count += 1;
                        }
                    }
                    break;
                }
            }
        }
        count
    }

    pub fn get_inventory_item_count(json_value: &Value, upgrade_code: &str) -> i32 {
        let (list_id_opt, array_id) = match Self::find_inventory_refs(json_value) {
            Ok(refs) => refs,
            Err(_) => return 0,
        };
        let size_limit = list_id_opt.map(|lid| Self::get_list_size(json_value, lid));
        let values = match json_value["records"][&array_id.to_string()]["values"].as_array() {
            Some(v) => v,
            None => return 0,
        };
        let mut count = 0i32;
        let mut logical_idx = 0i32;
        for entry in values {
            if let Some(limit) = size_limit {
                if logical_idx >= limit {
                    break;
                }
            }
            match entry["type"].as_str() {
                Some("Reference") => {
                    if let Some(ref_id) = entry["id"].as_i64() {
                        if Self::read_upgrade_record_name(json_value, ref_id) == upgrade_code {
                            count += 1;
                        }
                    }
                    logical_idx += 1;
                }
                Some("NullMultiple") => {
                    let c = entry["count"].as_i64().unwrap_or(1) as i32;
                    logical_idx += c;
                }
                Some("Null") => {
                    logical_idx += 1;
                }
                _ => {
                    logical_idx += 1;
                }
            }
        }
        count
    }

    pub fn get_total_item_count(json_value: &Value, upgrade_code: &str) -> i32 {
        Self::get_hero_item_count(json_value, upgrade_code)
            + Self::get_inventory_item_count(json_value, upgrade_code)
    }

    pub fn add_item_to_inventory(json_value: &mut Value, upgrade_code: &str) -> Result<()> {
        let (list_id_opt, array_id) = Self::find_inventory_refs(json_value)?;
        let current_size = list_id_opt.map(|lid| Self::get_list_size(json_value, lid)).unwrap_or(0);

        let slot_info: Option<(usize, i32)>;
        {
            let values = json_value["records"][&array_id.to_string()]["values"]
                .as_array()
                .ok_or_else(|| anyhow!("BinaryArray {} has no values array", array_id))?;
            let mut logical_idx = 0i32;
            let mut found: Option<(usize, i32)> = None;
            for (i, entry) in values.iter().enumerate() {
                match entry["type"].as_str() {
                    Some("Reference") => {
                        logical_idx += 1;
                    }
                    Some("NullMultiple") => {
                        if logical_idx >= current_size {
                            let c = entry["count"].as_i64().unwrap_or(1) as i32;
                            found = Some((i, c));
                            break;
                        }
                        let c = entry["count"].as_i64().unwrap_or(1) as i32;
                        logical_idx += c;
                    }
                    Some("Null") => {
                        if logical_idx >= current_size {
                            found = Some((i, 1));
                            break;
                        }
                        logical_idx += 1;
                    }
                    _ => {
                        logical_idx += 1;
                    }
                }
            }
            slot_info = found;
        }



        let (slot_index, null_multiple_count) = if let Some(slot) = slot_info {
            slot
        } else {
            let values = json_value["records"][&array_id.to_string()]["values"]
                .as_array_mut()
                .ok_or_else(|| anyhow!("BinaryArray {} has no values array for expansion", array_id))?;
            let new_idx = values.len();
            values.push(serde_json::json!({"type": "NullMultiple", "count": 256}));
            info!("Expanded inventory BinaryArray {} by appending NullMultiple(256) at index {}", array_id, new_idx);
            (new_idx, 256i32)
        };

        let (class_type_id, class_type_name) =
            Self::find_serializable_hero_upgrade_class_type(json_value)
                .ok_or_else(|| anyhow!("找不到SerializableHeroUpgrade 类型信息"))?;

        let max_id = Self::find_max_record_id(json_value);
        let upgrade_id = max_id + 1;
        let upgrade_id_str = upgrade_id.to_string();

        let initial_level = Self::get_item_initial_level(upgrade_code);



        let extra_fields = Self::get_item_required_fields(upgrade_code);
        let mut members = vec![
            serde_json::json!({"name": "name",  "type": "String", "value": upgrade_code}),
            serde_json::json!({"name": "level", "type": "Int32",  "value": initial_level}),
        ];
        for (_, field_value) in extra_fields {
            members.push(field_value);
        }
        let upgrade_record = serde_json::json!({
            "type": "Class",
            "class_type_id": class_type_id,
            "class_type_name": class_type_name,
            "members": members
        });
        json_value["records"][&upgrade_id_str] = upgrade_record;

        let metadata = Self::get_upgrade_record_metadata_template(json_value);
        if let Some(meta_obj) = json_value.get_mut("record_metadata").and_then(|m| m.as_object_mut()) {
            meta_obj.insert(upgrade_id_str.clone(), metadata);
        }

        if let Some(order) = json_value.get_mut("record_order").and_then(|v| v.as_array_mut()) {
            let upgrade_id_i32 = i32::try_from(upgrade_id)
                .map_err(|_| anyhow!("record ID {} exceeds i32 range", upgrade_id))?;
            order.push(Value::Number(upgrade_id_i32.into()));
        }

        let new_entry = serde_json::json!({"type": "Reference", "id": upgrade_id});
        if let Some(values) = json_value["records"][&array_id.to_string()]["values"].as_array_mut() {
            if null_multiple_count > 1 {
                values[slot_index] = new_entry;
                let residual =
                    serde_json::json!({"type": "NullMultiple", "count": null_multiple_count - 1});
                values.insert(slot_index + 1, residual);
            } else {
                values[slot_index] = new_entry;
            }
        }

        if let Some(lid) = list_id_opt {
            Self::set_list_size(json_value, lid, current_size + 1)?;
        }

        info!(
            "Added {} to inventory: upgrade_id={}, slot_index={}",
            upgrade_code, upgrade_id, slot_index
        );
        if null_multiple_count <= 255 {
            warn!(
                "⚠️背包物品已写入JSON，但存储时可能失败：\
                BinaryArray 槽位原来是NullMultiple(count={})（ 字节），\
                替换为Reference（ 字节）会导致记录字节长度变化。",
                null_multiple_count
            );
        }
        Ok(())
    }

    pub fn remove_item_from_inventory(json_value: &mut Value, upgrade_code: &str) -> Result<()> {
        let (list_id_opt, array_id) = Self::find_inventory_refs(json_value)?;
        let current_size =
            list_id_opt.map(|lid| Self::get_list_size(json_value, lid)).unwrap_or(0);

        let item_json_index: usize;
        {
            let values = json_value["records"][&array_id.to_string()]["values"]
                .as_array()
                .ok_or_else(|| anyhow!("BinaryArray {} has no values array", array_id))?;
            let mut logical_idx = 0i32;
            let mut found: Option<usize> = None;
            for (i, entry) in values.iter().enumerate() {
                if logical_idx >= current_size {
                    break;
                }
                match entry["type"].as_str() {
                    Some("Reference") => {
                        if let Some(ref_id) = entry["id"].as_i64() {
                            if Self::read_upgrade_record_name(json_value, ref_id) == upgrade_code {
                                found = Some(i);
                                break;
                            }
                        }
                        logical_idx += 1;
                    }
                    Some("NullMultiple") => {
                        let c = entry["count"].as_i64().unwrap_or(1) as i32;
                        logical_idx += c;
                    }
                    Some("Null") => {
                        logical_idx += 1;
                    }
                    _ => {
                        logical_idx += 1;
                    }
                }
            }
            item_json_index =
                found.ok_or_else(|| anyhow!("背包中没有{} 可移除", upgrade_code))?;
        }

        let last_item_json_index: usize;
        {
            let values = json_value["records"][&array_id.to_string()]["values"]
                .as_array()
                .ok_or_else(|| anyhow!("BinaryArray {} has no values array", array_id))?;
            let mut logical_idx = 0i32;
            let target_logical = current_size - 1;
            let mut found: Option<usize> = None;
            for (i, entry) in values.iter().enumerate() {
                if logical_idx > target_logical {
                    break;
                }
                match entry["type"].as_str() {
                    Some("Reference") => {
                        if logical_idx == target_logical {
                            found = Some(i);
                            break;
                        }
                        logical_idx += 1;
                    }
                    Some("NullMultiple") => {
                        let c = entry["count"].as_i64().unwrap_or(1) as i32;
                        if logical_idx + c > target_logical {
                            break;
                        }
                        logical_idx += c;
                    }
                    Some("Null") => {
                        if logical_idx == target_logical {
                            found = Some(i);
                            break;
                        }
                        logical_idx += 1;
                    }
                    _ => {
                        logical_idx += 1;
                    }
                }
            }
            last_item_json_index =
                found.ok_or_else(|| anyhow!("找不到背包最后一个有效槽位"))?;
        }

        if let Some(values) =
            json_value["records"][&array_id.to_string()]["values"].as_array_mut()
        {
            if item_json_index != last_item_json_index {
                values.swap(item_json_index, last_item_json_index);
            }
            values[last_item_json_index] =
                serde_json::json!({"type": "NullMultiple", "count": 256});
        }

        if let Some(lid) = list_id_opt {
            Self::set_list_size(json_value, lid, current_size - 1)?;
        }

        info!("Removed {} from inventory, new size={}", upgrade_code, current_size - 1);
        Ok(())
    }

    pub fn increment_item_count(json_value: &mut Value, upgrade_code: &str) -> Result<i32> {
        Self::add_item_to_inventory(json_value, upgrade_code)?;
        Ok(Self::get_total_item_count(json_value, upgrade_code))
    }

    pub fn decrement_item_count(json_value: &mut Value, upgrade_code: &str) -> Result<i32> {
        Self::remove_item_from_inventory(json_value, upgrade_code)?;
        Ok(Self::get_total_item_count(json_value, upgrade_code))
    }

    pub fn set_item_count(
        json_value: &mut Value,
        upgrade_code: &str,
        target: i32,
    ) -> Result<()> {
        if target < 0 {
            return Err(anyhow!("{} 数量不能为负数", upgrade_code));
        }
        let current = Self::get_total_item_count(json_value, upgrade_code);
        if target > current {
            for _ in 0..(target - current) {
                Self::increment_item_count(json_value, upgrade_code)?;
            }
        } else if target < current {
            for _ in 0..(current - target) {
                Self::decrement_item_count(json_value, upgrade_code)?;
            }
        }
        Ok(())
    }

    item_count_shortcuts!(@dead_code bomb, BOMB_UPGRADE_CODE);
    item_count_shortcuts!(@dead_code mine, MINE_UPGRADE_CODE);
    item_count_shortcuts!(@dead_code philosophers_stone, PHILOSOPHERS_STONE_UPGRADE_CODE);
    item_count_shortcuts!(@dead_code size, SIZE_UPGRADE_CODE);
    item_count_shortcuts!(@dead_code warhammer, WARHAMMER_UPGRADE_CODE);
    item_count_shortcuts!(@dead_code cornucopia, CORNUCOPIA_UPGRADE_CODE);
    item_count_shortcuts!(@dead_code war_horn, WAR_HORN_UPGRADE_CODE);

    pub fn get_all_inventory_items(json_value: &Value) -> Vec<(String, i32)> {
        let (list_id_opt, array_id) = match Self::find_inventory_refs(json_value) {
            Ok(refs) => refs,
            Err(_) => return Vec::new(),
        };
        let size_limit = list_id_opt.map(|lid| Self::get_list_size(json_value, lid));
        let values = match json_value["records"][&array_id.to_string()]["values"].as_array() {
            Some(v) => v,
            None => return Vec::new(),
        };

        let mut counts: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
        let mut logical_idx = 0i32;

        for entry in values {
            if let Some(limit) = size_limit {
                if logical_idx >= limit {
                    break;
                }
            }
            match entry["type"].as_str() {
                Some("Reference") => {
                    if let Some(ref_id) = entry["id"].as_i64() {
                        let name = Self::read_upgrade_record_name(json_value, ref_id);
                        if !name.is_empty() {
                            *counts.entry(name).or_insert(0) += 1;
                        }
                    }
                    logical_idx += 1;
                }
                Some("NullMultiple") => {
                    let c = entry["count"].as_i64().unwrap_or(1) as i32;
                    logical_idx += c;
                }
                Some("Null") => {
                    logical_idx += 1;
                }
                _ => {
                    logical_idx += 1;
                }
            }
        }

        let mut result: Vec<(String, i32)> = counts.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }

    pub fn get_undeclared_inventory_items(json_value: &Value) -> Vec<(String, i32)> {
        const KNOWN_ITEMS: &[&str] = &[
            GRAIL_UPGRADE_CODE,
            BOMB_UPGRADE_CODE,
            MINE_UPGRADE_CODE,
            PHILOSOPHERS_STONE_UPGRADE_CODE,
            SIZE_UPGRADE_CODE,
            WARHAMMER_UPGRADE_CODE,
            CORNUCOPIA_UPGRADE_CODE,
            WAR_HORN_UPGRADE_CODE,
        ];
        Self::get_all_inventory_items(json_value)
            .into_iter()
            .filter(|(name, _)| !KNOWN_ITEMS.contains(&name.as_str()))
            .collect()
    }

    pub fn add_custom_item_to_inventory(json_value: &mut Value, upgrade_code: &str) -> Result<()> {
        let current_total = Self::get_total_inventory_count(json_value);
        if current_total >= 20 {
            return Err(anyhow!("❌背包已满（{}/20），无法继续添加装备", current_total));
        }
        Self::add_item_to_inventory(json_value, upgrade_code)
    }

    pub fn get_total_inventory_count(json_value: &Value) -> i32 {
        Self::get_all_inventory_items(json_value)
            .iter()
            .map(|(_, count)| count)
            .sum()
    }

    #[allow(dead_code)]
    pub fn replace_inventory_item(json_value: &mut Value, old_code: &str, new_code: &str) -> Result<i32> {
        let (_, array_id) = Self::find_inventory_refs(json_value)?;
        
        // First pass: collect all ref_ids that need to be replaced
        let mut ref_ids_to_replace: Vec<i64> = Vec::new();
        {
            if let Some(values) = json_value["records"][&array_id.to_string()]["values"].as_array() {
                for entry in values.iter() {
                    if entry["type"].as_str() == Some("Reference") {
                        if let Some(ref_id) = entry["id"].as_i64() {
                            let ref_id_str = ref_id.to_string();
                            if let Some(record) = json_value["records"][&ref_id_str].as_object() {
                                if let Some(members) = record.get("members").and_then(|m| m.as_array()) {
                                    for member in members.iter() {
                                        if member.get("name").and_then(|n| n.as_str()) == Some("name") {
                                            if member.get("value").and_then(|v| v.as_str()) == Some(old_code) {
                                                ref_ids_to_replace.push(ref_id);
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Second pass: perform the replacements
        let mut count = 0;
        for ref_id in ref_ids_to_replace {
            let ref_id_str = ref_id.to_string();
            if let Some(record) = json_value["records"][&ref_id_str].as_object_mut() {
                if let Some(members) = record.get_mut("members").and_then(|m| m.as_array_mut()) {
                    for member in members.iter_mut() {
                        if member.get("name").and_then(|n| n.as_str()) == Some("name") {
                            if member.get("value").and_then(|v| v.as_str()) == Some(old_code) {
                                member["value"] = Value::String(new_code.to_string());
                                count += 1;
                            }
                        }
                    }
                }
            }
        }

        if count == 0 {
            return Err(anyhow!("未找到装备 {}", old_code));
        }

        info!("背包中已将 {}(× {}) 替换为 {}", old_code, count, new_code);
        Ok(count)
    }
}

#[derive(Debug, Clone, Default)]
pub struct HeroDetails {
    pub key: String,
    pub id: i32,
    pub coins: i32,
    pub hue: f32,
    pub has_crown: bool,
    pub crown_style: Option<String>,
    pub soldiers_lost: i32,
    pub max_soldiers: i32,
    pub class_upgrade_ref: Option<String>,
    pub item_upgrade_ref: Option<String>,
    pub trait_upgrade_ref: Option<String>,
    pub skill_upgrade_ref: Option<String>,
    pub upgrade_ref: Option<String>,
    pub class_info: Option<UpgradeInfo>,
    pub item_info: Option<UpgradeInfo>,
    pub trait_info: Option<UpgradeInfo>,
    pub skill_info: Option<UpgradeInfo>,
    pub upgrade_info: Option<UpgradeInfo>,
}

#[derive(Debug, Clone, Default)]
pub struct UpgradeInfo {
    pub name: String,
    pub level: i32,
    pub record_id: String,
    pub field_name: String,
}

impl HeroDetails {
    #[allow(dead_code)]
    pub fn class_display(&self) -> String {
        self.class_info.as_ref()
            .map(|u| format!("{}(L{})", u.name, u.level))
            .unwrap_or_else(|| "No Class".to_string())
    }

    #[allow(dead_code)]
    pub fn item_display(&self) -> String {
        self.item_info.as_ref()
            .map(|u| format!("{}(L{})", u.name, u.level))
            .unwrap_or_else(|| "No Item".to_string())
    }

    #[allow(dead_code)]
    pub fn trait_display(&self) -> String {
        self.trait_info.as_ref()
            .map(|u| u.name.clone())
            .unwrap_or_else(|| "No Trait".to_string())
    }

    #[allow(dead_code)]
    pub fn skill_display(&self) -> String {
        self.skill_info.as_ref()
            .map(|u| format!("{}(L{})", u.name, u.level))
            .unwrap_or_else(|| "No Skill".to_string())
    }

    #[allow(dead_code)]
    pub fn upgrade_display(&self) -> String {
        self.upgrade_info.as_ref()
            .map(|u| u.name.clone())
            .unwrap_or_else(|| "No Upgrade".to_string())
    }

    #[allow(dead_code)]
    pub fn crown_display(&self) -> String {
        if self.has_crown {
            "✔已启用".to_string()
        } else {
            "✗未启用".to_string()
        }
    }
}

