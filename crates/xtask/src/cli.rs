use std::path::PathBuf;

use clap::Subcommand;

#[derive(clap::Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand)]
pub enum Cmd {
    /// 全量或指定用例的语料验证,产出 data/ 五份数据 CSV 与 verdict.json
    Verify(VerifyArgs),
    /// 清空 /tmp/tsox-verify 归档
    Clean,
    /// 在 base 检出内把当前水位立为波起点快照(已存在则跳过,--force 覆盖)
    Snapshot {
        #[arg(long, default_value_t = false)]
        force: bool,
    },
    /// 隐藏子命令:单用例 worker 进程入口(由 verify 拉起,勿直接调用)
    #[command(name = "__worker", hide = true)]
    Worker(WorkerArgs),
}

#[derive(clap::Args)]
pub struct VerifyArgs {
    /// 用例清单文件,一行一个 key(compiler/foo.ts 或 foo.ts,# 注释);缺省全量
    #[arg(long, value_name = "FILE")]
    pub cases: Option<PathBuf>,

    /// 目标例 key(全量跑时填充 verdict.selected 的判定字段,供波验证消费)
    #[arg(long, value_name = "KEY")]
    pub target: Option<String>,

    /// 并发 worker 数
    #[arg(long, default_value_t = 12)]
    pub jobs: usize,

    /// 每个 worker 进程的内存上限(RLIMIT_AS,MB)
    #[arg(long, default_value_t = 4096)]
    pub mem_mb: u64,

    /// 单用例超时(秒),超时杀进程记 SKIP
    #[arg(long, default_value_t = 30)]
    pub case_timeout_secs: u64,

    /// 整轮超时(秒),到点停止派发并出判定
    #[arg(long, default_value_t = 2400)]
    pub round_timeout_secs: u64,

    /// Go reference .types 目录(锚点比对基准)
    #[arg(long, default_value = crate::paths::DEFAULT_GO_REF, value_name = "DIR")]
    pub go_ref: PathBuf,

    /// 波起点 base 快照所属检出根目录(将自动拼 data/;缺省自动探测:TSOX_BASE_DIR >
    /// main 分支所在 worktree;探测不到则归因退化为绝对水位)
    #[arg(long, value_name = "DIR")]
    pub base_dir: Option<PathBuf>,

    /// base 检出内强制重拍波起点 base 快照
    #[arg(long, default_value_t = false)]
    pub refresh_baseline: bool,

    /// 归档保留轮数(配合 --keep-days 启动清扫)
    #[arg(long, default_value_t = 20)]
    pub keep_runs: u32,

    /// 归档保留天数
    #[arg(long, default_value_t = 14)]
    pub keep_days: u32,
}

#[derive(clap::Args)]
pub struct WorkerArgs {
    /// 单例模式的目标用例;--resident 时空缺
    #[arg(long, requires = "out")]
    pub case: Option<PathBuf>,

    #[arg(long, requires = "case")]
    pub out: Option<PathBuf>,

    /// 常驻模式:从 stdin 逐行读请求,处理完回 DONE,EOF 退出
    #[arg(long, default_value_t = false)]
    pub resident: bool,
}
