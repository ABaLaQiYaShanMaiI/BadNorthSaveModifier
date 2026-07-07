/// ========== 升级字典模块 ==========
/// 本模块按游戏内容版本组织所有升级、特质、技能和兵种定义
/// 
/// 数据来源分类：
/// - 原版 (Original): Bad North 官方原版
/// - 旧恢复的战旗 (Old Grey Flag): 《旧恢复燃的战旗》DLC (已停支)
/// - 融合版 (Fusion): 《旧恢复燃的战旗 · 融合版》整合版
///
/// 组织层级：装备 → 特质 → 技能 → 兵种

// ==================== 数据模型定义 ====================

#[allow(dead_code)]
pub struct UpgradeEntry {
    pub code: &'static str,
    pub chinese_name: &'static str,
    pub initial_level: i32,
}

#[allow(dead_code)]
pub struct ClassEntry {
    pub code: &'static str,
    pub chinese_name: &'static str,
    pub initial_level: i32,
}

// ==================== 装备字典 (Items) ====================
// 装备为英雄提供一次性或多次性能力增强

/// 装备 · 原版
/// 游戏原生的8种装备（基础内容）
pub const ITEM_DICTIONARY_ORIGINAL: &[UpgradeEntry] = &[
    UpgradeEntry { code: "Hero_Upgrade_Bomb",              chinese_name: "炸弹",       initial_level: 0 },
    UpgradeEntry { code: "Hero_Upgrade_Cornucopia",        chinese_name: "雅贝那",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Upgrade_Grail",             chinese_name: "圣杯",       initial_level: 0 },
    UpgradeEntry { code: "Hero_Upgrade_Mine",              chinese_name: "地雷",       initial_level: 0 },
    UpgradeEntry { code: "Hero_Upgrade_PhilosophersStone", chinese_name: "贤者之石",   initial_level: 0 },
    UpgradeEntry { code: "Hero_Upgrade_Size",              chinese_name: "指挥之戒",   initial_level: 0 },
    UpgradeEntry { code: "Hero_Upgrade_Horn",           chinese_name: "战争号角",   initial_level: 0 },
    UpgradeEntry { code: "Hero_Upgrade_Warhammer",         chinese_name: "战锤",       initial_level: 0 },
];

/// 装备 · 融合版（新增）
/// 《旧恢复燃的战旗 · 融合版》中新增的装备
#[allow(dead_code)]
pub const ITEM_DICTIONARY_FUSION_NEW: &[UpgradeEntry] = &[
    UpgradeEntry { code: "Hero_Item_Charge",       chinese_name: "盾冲",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Item_DeathZone",    chinese_name: "死亡区域", initial_level: 0 },
    UpgradeEntry { code: "Hero_Item_FrontArmor", chinese_name: "正面护甲", initial_level: 0 },
    UpgradeEntry { code: "Hero_Item_SpeedUp",    chinese_name: "心灵加速器", initial_level: 0 },
];

/// 装备 · 魔改版（专属集合）
/// 魔改版专属的装备集合
pub const ITEM_DICTIONARY_MOD_VERSION: &[UpgradeEntry] = &[
    UpgradeEntry { code: "Hero_Item_Charge",       chinese_name: "盾冲",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Item_FrontArmor", chinese_name: "正面护甲", initial_level: 0 },
    UpgradeEntry { code: "Hero_Item_SpeedUp",    chinese_name: "心灵加速器", initial_level: 0 },
];

// ==================== 特质字典 (Traits) ====================
// 特质为英雄提供被动技能或属性加成

/// 特质 · 原版
/// 游戏原生的13种特质（基础内容）
pub const TRAIT_DICTIONARY_ORIGINAL: &[UpgradeEntry] = &[
    UpgradeEntry { code: "Hero_Trait_BluntWeapons",      chinese_name: "重型武器",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_CheaperItems",      chinese_name: "收藏家",       initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_CheaperSkills",     chinese_name: "技艺熟手",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_ExtraArmor",        chinese_name: "钢铁之躯",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_ExtraUnit",         chinese_name: "深受爱戴",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_ExtraUses",         chinese_name: "重型负载",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_Fast",              chinese_name: "加速行进",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_FastReplenish",     chinese_name: "振奋人心的演说", initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_Giant",             chinese_name: "山岳",         initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_NoFlee",            chinese_name: "无畏",         initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_SharpWeapons",      chinese_name: "尖锐武器",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_ShortCooldown",     chinese_name: "精力充沛",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_Sturdy",            chinese_name: "脚踏实地",     initial_level: 0 },
];

/// 特质 · 旧恢复燃的战旗（已停支内容）
/// 《旧恢复燃的战旗》独占的特质（仅在旧版DLC中有效，融合版已删除）
/// 不推荐用于新游戏，这些特质在融合版中被重新设计
#[allow(dead_code)]
pub const TRAIT_DICTIONARY_OLDGREY_FLAG_LEGACY: &[UpgradeEntry] = &[
    UpgradeEntry { code: "Hero_Trait_AxeThrower",        chinese_name: "掷斧手",       initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_CheaperClass",      chinese_name: "迅捷精英",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_Regenerative",      chinese_name: "追猎",         initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_Thorns",            chinese_name: "荆棘",         initial_level: 0 },
];

/// 特质 · 融合版（新增 & 重设计）
/// 《旧恢复燃的战旗 · 融合版》中新增的特质
/// 包含全新特质(11个) + 旧版DLC升级版本(3个) + 融合版专属(7个) = 21个
#[allow(dead_code)]
pub const TRAIT_DICTIONARY_FUSION_NEW: &[UpgradeEntry] = &[
    // 全新特质 (11个)
    UpgradeEntry { code: "Hero_Trait_AxeThrower",        chinese_name: "投斧大队",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_StunCommander",     chinese_name: "眩晕大师",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_Demolitionist",     chinese_name: "投弹专家",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_SlowAura",          chinese_name: "减速光环",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_AreaCommander",     chinese_name: "力场行进",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_LoneWolf",          chinese_name: "孤狼",         initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_ApocalypseKnight",  chinese_name: "天启三骑士",   initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_Ranger",            chinese_name: "游骑兵",       initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_ShieldOfEmpire",    chinese_name: "帝国之盾",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_Ashtar",            chinese_name: "阿斯塔特",     initial_level: 0 },
    // 旧版DLC升级版本 (3个)
    UpgradeEntry { code: "Hero_Trait_Thorns",            chinese_name: "荆棘",         initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_CheaperClass",      chinese_name: "快速精英",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_Regenerative",      chinese_name: "医疗训练",     initial_level: 0 },
    // 融合版专属新特质 (5个)
    UpgradeEntry { code: "Hero_Trait_Jumper",            chinese_name: "跳劈大队",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_Creeper",           chinese_name: "短人部队",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_Flyer",             chinese_name: "神鹰",         initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_Titan",             chinese_name: "泰坦",         initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_Slash",             chinese_name: "横扫之刃",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_Yuri",              chinese_name: "心灵精英",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_UltimateSquad",     chinese_name: "终极部队",     initial_level: 0 },
];

/// 特质 · 魔改版（专属集合）
/// 魔改版专属的特质集合（11个）
pub const TRAIT_DICTIONARY_MOD_VERSION: &[UpgradeEntry] = &[
    UpgradeEntry { code: "Hero_Trait_AxeThrower",        chinese_name: "投斧大队",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_Thorns",            chinese_name: "荆棘",         initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_Regenerative",      chinese_name: "医疗训练",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_CheaperClass",      chinese_name: "快速精通",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_Jumper",            chinese_name: "跳劈大队",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_Creeper",           chinese_name: "短人部队",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_Flyer",             chinese_name: "神鹰",         initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_Titan",             chinese_name: "泰坦",         initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_Slash",             chinese_name: "横扫之刃",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_Yuri",              chinese_name: "心灵精英",     initial_level: 0 },
    UpgradeEntry { code: "Hero_Trait_UltimateSquad",     chinese_name: "终极部队",     initial_level: 0 },
];

// 向后兼容别名
// 注：TRAIT_DICTIONARY_FUSION 包含原版特质(13) + 融合版新增(21) = 占用总体位置
pub const TRAIT_DICTIONARY: &[UpgradeEntry] = TRAIT_DICTIONARY_ORIGINAL;
#[allow(dead_code)]
pub const TRAIT_DICTIONARY_OLDGREY_FLAG: &[UpgradeEntry] = TRAIT_DICTIONARY_OLDGREY_FLAG_LEGACY;
#[allow(dead_code)]
pub const TRAIT_DICTIONARY_FUSION: &[UpgradeEntry] = TRAIT_DICTIONARY_FUSION_NEW;

// 装备别名（向后兼容）
pub const UPGRADE_DICTIONARY: &[UpgradeEntry] = ITEM_DICTIONARY_ORIGINAL;
#[allow(dead_code)]
pub const ITEM_DICTIONARY_FUSION: &[UpgradeEntry] = ITEM_DICTIONARY_FUSION_NEW;

// ==================== 技能字典 (Skills) ====================
// 技能为英雄提供主动释放的特殊能力

/// 技能 · 原版（唯一版本）
/// 游戏原生的6种技能（对应3个兵种职业）
#[allow(dead_code)]
pub const SKILL_DICTIONARY: &[UpgradeEntry] = &[
    UpgradeEntry { code: "Hero_Skill_Charge",       chinese_name: "冲锋",     initial_level: 1 },
    UpgradeEntry { code: "Hero_Skill_ArrowRain",    chinese_name: "箭雨",     initial_level: 1 },
    UpgradeEntry { code: "Hero_Skill_ShieldWall",   chinese_name: "盾墙",     initial_level: 1 },
    UpgradeEntry { code: "Hero_Skill_Rally",        chinese_name: "集结",     initial_level: 1 },
    UpgradeEntry { code: "Hero_Skill_Berserker",    chinese_name: "狂战士",   initial_level: 1 },
    UpgradeEntry { code: "Hero_Skill_Volley",       chinese_name: "齐射",     initial_level: 1 },
];

// ==================== 兵种字典 (Classes) ====================
// 兵种为英雄的职业属性，决定作战风格和技能

/// 兵种 · 原版（唯一版本）
/// 游戏原生的3种兵种（基础内容，各版本统一）
#[allow(dead_code)]
pub const CLASS_DICTIONARY: &[ClassEntry] = &[
    ClassEntry { code: "Hero_Class_Infantry", chinese_name: "步兵",   initial_level: 1 },
    ClassEntry { code: "Hero_Class_Archers",  chinese_name: "弓箭手", initial_level: 1 },
    ClassEntry { code: "Hero_Class_Pikemen",  chinese_name: "长矛手", initial_level: 1 },
];

// ==================== 统计信息 ====================
// 方便查看各版本的内容量差异

#[cfg(test)]
mod stats {
    use super::*;

    pub fn print_stats() {
        println!("=== 升级字典统计信息 ===");
        println!("装备 · 原版: {} 个", ITEM_DICTIONARY_ORIGINAL.len());
        println!("装备 · 融合新增: {} 个", ITEM_DICTIONARY_FUSION_NEW.len());
        println!("装备 · 完整: {} 个（原版 + 融合新增）", ITEM_DICTIONARY_ORIGINAL.len() + ITEM_DICTIONARY_FUSION_NEW.len());
        println!("");
        println!("特质 · 原版: {} 个", TRAIT_DICTIONARY_ORIGINAL.len());
        println!("特质 · 旧版DLC: {} 个（已停支）", TRAIT_DICTIONARY_OLDGREY_FLAG_LEGACY.len());
        println!("特质 · 融合版新增: {} 个（原版 + 新增）", TRAIT_DICTIONARY_FUSION_NEW.len());
        println!("");
        println!("技能 · 原版: {} 个", SKILL_DICTIONARY.len());
        println!("兵种 · 原版: {} 个", CLASS_DICTIONARY.len());
    }
}
