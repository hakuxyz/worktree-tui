/// Modal dialog state for interactive input.
#[derive(Debug, Clone)]
pub enum Dialog {
    None,
    AddWorktree(AddWorktreeDialog),
    CreateBranch(CreateBranchDialog),
    RenameBranch(RenameBranchDialog),
    ConfirmRemove(String), // worktree path
}

#[derive(Debug, Clone)]
pub struct AddWorktreeDialog {
    pub path: String,
    pub branch: String,
    pub field: InputField, // 0 = path, 1 = branch
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateBranchDialog {
    pub name: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RenameBranchDialog {
    pub old_name: String,
    pub new_name: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputField {
    Path,
    Branch,
}

impl Dialog {
    pub fn add_worktree() -> Self {
        Dialog::AddWorktree(AddWorktreeDialog {
            path: String::new(),
            branch: String::new(),
            field: InputField::Path,
            error: None,
        })
    }

    pub fn create_branch() -> Self {
        Dialog::CreateBranch(CreateBranchDialog {
            name: String::new(),
            error: None,
        })
    }
}

impl Default for Dialog {
    fn default() -> Self {
        Dialog::None
    }
}
