use crate::config::DotConfig;
use crate::error::ConfigError;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

pub struct SetupWizard;

impl SetupWizard {
    /// 运行交互式设置向导
    pub async fn run() -> Result<(), ConfigError> {
        println!();
        println!("🔧 dot 初始化设置向导");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();
        
        // 步骤 1: 检查 Git 配置
        Self::check_git_config()?;
        
        // 步骤 2: 获取用户的 GitHub 用户名
        let github_username = Self::get_github_username()?;
        
        // 步骤 3: 询问要使用的组织
        let organization = Self::prompt_organization(&github_username)?;
        
        // 步骤 4: 询问 GitHub Token（可选）
        let github_token = Self::prompt_github_token()?;
        
        // 步骤 5: 创建配置文件
        Self::create_config(&organization, github_token.as_deref()).await?;
        
        // 步骤 6: 检查并创建 .index 仓库
        Self::setup_index_repository(&organization).await?;
        
        println!();
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("✅ 设置完成！");
        println!();
        println!("现在你可以使用以下命令：");
        println!("  dot init .kiro          # 初始化隐藏目录");
        println!("  dot status              # 查看状态");
        println!("  dot add .               # 添加文件");
        println!("  dot commit -m \"msg\"     # 提交更改");
        println!("  dot push                # 推送到远程");
        println!();
        
        Ok(())
    }
    
    /// 检查 Git 配置
    fn check_git_config() -> Result<(), ConfigError> {
        println!("📋 步骤 1/5: 检查 Git 配置");
        println!();
        
        // 检查 git 是否安装
        let git_version = Command::new("git")
            .arg("--version")
            .output();
            
        match git_version {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                println!("   ✓ Git 已安装: {}", version.trim());
            }
            _ => {
                println!("   ✗ Git 未安装或不在 PATH 中");
                println!("   请先安装 Git: https://git-scm.com/");
                return Err(ConfigError::IoError(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Git not found"
                )));
            }
        }
        
        // 检查 git user.name
        let user_name = Command::new("git")
            .args(["config", "--global", "user.name"])
            .output();
            
        match user_name {
            Ok(output) if output.status.success() && !output.stdout.is_empty() => {
                let name = String::from_utf8_lossy(&output.stdout);
                println!("   ✓ Git 用户名: {}", name.trim());
            }
            _ => {
                println!("   ✗ Git 用户名未配置");
                println!("   请运行: git config --global user.name \"Your Name\"");
            }
        }
        
        // 检查 git user.email
        let user_email = Command::new("git")
            .args(["config", "--global", "user.email"])
            .output();
            
        match user_email {
            Ok(output) if output.status.success() && !output.stdout.is_empty() => {
                let email = String::from_utf8_lossy(&output.stdout);
                println!("   ✓ Git 邮箱: {}", email.trim());
            }
            _ => {
                println!("   ✗ Git 邮箱未配置");
                println!("   请运行: git config --global user.email \"your@email.com\"");
            }
        }
        
        println!();
        Ok(())
    }
    
    /// 获取 GitHub 用户名
    fn get_github_username() -> Result<String, ConfigError> {
        println!("👤 步骤 2/5: 获取 GitHub 用户名");
        println!();
        
        // 尝试从 git config 获取 GitHub 用户名
        let gh_user = Command::new("git")
            .args(["config", "--global", "github.user"])
            .output();
            
        let suggested_username = match gh_user {
            Ok(output) if output.status.success() && !output.stdout.is_empty() => {
                let username = String::from_utf8_lossy(&output.stdout).trim().to_string();
                println!("   发现 GitHub 用户名: {}", username);
                Some(username)
            }
            _ => {
                // 尝试从 gh cli 获取
                let gh_cli = Command::new("gh")
                    .args(["api", "user", "-q", ".login"])
                    .output();
                    
                match gh_cli {
                    Ok(output) if output.status.success() && !output.stdout.is_empty() => {
                        let username = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        println!("   从 GitHub CLI 获取用户名: {}", username);
                        Some(username)
                    }
                    _ => None
                }
            }
        };
        
        let username = if let Some(suggested) = suggested_username {
            print!("   使用此用户名? [Y/n]: ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim().to_lowercase();
            
            if input.is_empty() || input == "y" || input == "yes" {
                suggested
            } else {
                Self::prompt_input("   请输入你的 GitHub 用户名: ")?
            }
        } else {
            Self::prompt_input("   请输入你的 GitHub 用户名: ")?
        };
        
        println!();
        Ok(username)
    }
    
    /// 询问要使用的组织
    fn prompt_organization(github_username: &str) -> Result<String, ConfigError> {
        println!("🏢 步骤 3/6: 选择 GitHub 组织");
        println!();
        println!("   dot 需要一个 GitHub 组织来存储隐藏仓库。");
        println!("   你可以使用自己的用户名作为组织（个人账户），");
        println!("   或者使用你有写权限的组织。");
        println!();
        println!("   默认: {} (你的个人账户)", github_username);
        println!();
        
        print!("   请输入组织名称 [{}]: ", github_username);
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        let organization = if input.is_empty() {
            github_username.to_string()
        } else {
            input.to_string()
        };
        
        println!("   ✓ 将使用组织: {}", organization);
        println!();
        
        Ok(organization)
    }
    
    /// 询问 GitHub Token（可选）
    fn prompt_github_token() -> Result<Option<String>, ConfigError> {
        println!("🔑 步骤 4/6: 配置 GitHub Token（可选）");
        println!();
        println!("   GitHub Token 用于通过 API 创建远程仓库。");
        println!("   如果不配置，将使用 GitHub CLI (gh) 作为备选方案。");
        println!();
        println!("   获取 Token: https://github.com/settings/tokens");
        println!("   需要的权限: repo (Full control of private repositories)");
        println!();
        
        print!("   请输入 GitHub Token (留空跳过): ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.is_empty() {
            println!("   ⚠️  未配置 Token，将使用 GitHub CLI (gh) 创建仓库");
            println!("      请确保已运行 'gh auth login'");
            println!();
            return Ok(None);
        }
        
        // 简单验证 token 格式
        if input.starts_with("ghp_") || input.starts_with("github_pat_") || input.len() > 30 {
            println!("   ✓ GitHub Token 已配置");
            println!();
            Ok(Some(input.to_string()))
        } else {
            println!("   ⚠️  Token 格式可能不正确，但仍会保存");
            println!();
            Ok(Some(input.to_string()))
        }
    }
    
    /// 创建配置文件
    async fn create_config(organization: &str, github_token: Option<&str>) -> Result<(), ConfigError> {
        println!("📝 步骤 5/6: 创建配置文件");
        println!();
        
        let config_path = Self::config_file_path()?;
        
        // 检查配置文件是否已存在
        if config_path.exists() {
            println!("   发现已有配置文件: {}", config_path.display());
            print!("   是否覆盖? [y/N]: ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim().to_lowercase();
            
            if input != "y" && input != "yes" {
                println!("   保留现有配置");
                println!();
                return Ok(());
            }
        }
        
        // 创建配置目录
        if let Some(parent) = config_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        // 创建配置
        let config = DotConfig {
            authorized_organizations: vec![organization.to_string()],
            default_organization: Some(organization.to_string()),
            github_token: github_token.map(|s| s.to_string()),
        };
        
        let content = serde_json::to_string_pretty(&config)
            .map_err(|e| ConfigError::JsonError(e))?;
        tokio::fs::write(&config_path, content).await?;
        
        println!("   ✓ 配置文件已创建: {}", config_path.display());
        println!();
        
        Ok(())
    }
    
    /// 设置 .index 仓库
    async fn setup_index_repository(organization: &str) -> Result<(), ConfigError> {
        println!("📦 步骤 6/6: 设置索引仓库");
        println!();
        
        let dot_dir = Self::dot_dir()?;
        let index_path = dot_dir.join(".index");
        
        // 检查本地 .index 目录是否存在
        if index_path.exists() {
            println!("   发现本地索引目录: {}", index_path.display());
            println!("   ✓ 索引仓库已配置");
            println!();
            return Ok(());
        }
        
        // 尝试克隆远程 .index 仓库
        let remote_url = format!("git@github.com:{}/{}.git", organization, ".index");
        println!("   尝试克隆索引仓库: {}", remote_url);
        
        let clone_result = Command::new("git")
            .args(["clone", &remote_url, index_path.to_str().unwrap()])
            .output();
            
        match clone_result {
            Ok(output) if output.status.success() => {
                println!("   ✓ 索引仓库已克隆");
            }
            _ => {
                // 仓库不存在，需要创建
                println!("   索引仓库不存在，正在创建...");
                println!();
                println!("   ⚠️  请在 GitHub 上手动创建仓库:");
                println!("      1. 访问 https://github.com/new");
                println!("      2. Repository name: .index");
                println!("      3. Owner: {}", organization);
                println!("      4. 选择 Private");
                println!("      5. 勾选 \"Add a README file\"");
                println!("      6. 点击 \"Create repository\"");
                println!();
                print!("   创建完成后按 Enter 继续...");
                io::stdout().flush()?;
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                
                // 再次尝试克隆
                let retry_result = Command::new("git")
                    .args(["clone", &remote_url, index_path.to_str().unwrap()])
                    .output();
                    
                match retry_result {
                    Ok(output) if output.status.success() => {
                        println!("   ✓ 索引仓库已克隆");
                    }
                    _ => {
                        println!("   ⚠️  无法克隆索引仓库，请稍后手动运行 'dot setup' 重试");
                        println!("      或者手动克隆: git clone {} {}", remote_url, index_path.display());
                    }
                }
            }
        }
        
        println!();
        Ok(())
    }
    
    /// 获取用户输入
    fn prompt_input(prompt: &str) -> Result<String, ConfigError> {
        print!("{}", prompt);
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let input = input.trim().to_string();
        if input.is_empty() {
            return Err(ConfigError::IoError(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Input cannot be empty"
            )));
        }
        
        Ok(input)
    }
    
    fn config_file_path() -> Result<PathBuf, ConfigError> {
        let home = dirs::home_dir().ok_or(ConfigError::HomeDirectoryNotFound)?;
        Ok(home.join(".dot").join("dot.conf"))
    }
    
    fn dot_dir() -> Result<PathBuf, ConfigError> {
        let home = dirs::home_dir().ok_or(ConfigError::HomeDirectoryNotFound)?;
        Ok(home.join(".dot"))
    }
}
