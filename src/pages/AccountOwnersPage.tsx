import { useState, useEffect, useCallback } from "react";
import {
  Box,
  Typography,
  Button,
  List,
  ListItem,
  ListItemText,
  IconButton,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Alert,
  CircularProgress,
} from "@mui/material";
import AddIcon from "@mui/icons-material/Add";
import EditIcon from "@mui/icons-material/Edit";
import DeleteIcon from "@mui/icons-material/Delete";
import {
  listAccountOwners,
  createAccountOwner,
  updateAccountOwner,
  deleteAccountOwner,
} from "../api/account_owner";
import { AccountOwner } from "../types";

/**
 * 归属人管理页面
 * 支持新增、编辑、删除归属人
 */
function AccountOwnersPage() {
  const [owners, setOwners] = useState<AccountOwner[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // 新增/编辑对话框状态
  const [dialogOpen, setDialogOpen] = useState(false);
  const [dialogMode, setDialogMode] = useState<"create" | "edit">("create");
  const [editingOwner, setEditingOwner] = useState<AccountOwner | null>(null);
  const [nameInput, setNameInput] = useState("");
  const [saving, setSaving] = useState(false);

  // 删除确认对话框状态
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [deletingOwner, setDeletingOwner] = useState<AccountOwner | null>(null);
  const [deleting, setDeleting] = useState(false);

  /** 加载归属人列表 */
  const loadOwners = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await listAccountOwners();
      setOwners(data);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadOwners();
  }, [loadOwners]);

  /** 点击新增 */
  const handleCreate = () => {
    setDialogMode("create");
    setEditingOwner(null);
    setNameInput("");
    setDialogOpen(true);
  };

  /** 点击编辑 */
  const handleEdit = (owner: AccountOwner) => {
    setDialogMode("edit");
    setEditingOwner(owner);
    setNameInput(owner.name);
    setDialogOpen(true);
  };

  /** 保存（新增或编辑） */
  const handleSave = async () => {
    const trimmed = nameInput.trim();
    if (!trimmed) return;
    setSaving(true);
    setError(null);
    try {
      if (dialogMode === "create") {
        await createAccountOwner(trimmed);
      } else if (editingOwner) {
        await updateAccountOwner(editingOwner.id, trimmed);
      }
      setDialogOpen(false);
      await loadOwners();
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  };

  /** 点击删除 */
  const handleDeleteClick = (owner: AccountOwner) => {
    setDeletingOwner(owner);
    setDeleteDialogOpen(true);
  };

  /** 确认删除 */
  const handleDeleteConfirm = async () => {
    if (!deletingOwner) return;
    setDeleting(true);
    setError(null);
    try {
      await deleteAccountOwner(deletingOwner.id);
      setDeleteDialogOpen(false);
      await loadOwners();
    } catch (e) {
      setError(String(e));
    } finally {
      setDeleting(false);
    }
  };

  return (
    <Box>
      <Box
        sx={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          mb: 3,
        }}
      >
        <Typography variant="h4">归属人管理</Typography>
        <Button
          variant="contained"
          startIcon={<AddIcon />}
          onClick={handleCreate}
        >
          新增归属人
        </Button>
      </Box>

      {error && (
        <Alert severity="error" sx={{ mb: 2 }} onClose={() => setError(null)}>
          {error}
        </Alert>
      )}

      {loading ? (
        <Box sx={{ display: "flex", justifyContent: "center", py: 4 }}>
          <CircularProgress />
        </Box>
      ) : owners.length === 0 ? (
        <Typography color="text.secondary" sx={{ py: 2 }}>
          暂无归属人，请点击"新增归属人"添加
        </Typography>
      ) : (
        <List>
          {owners.map((owner) => (
            <ListItem
              key={owner.id}
              divider
              secondaryAction={
                <>
                  <IconButton
                    onClick={() => handleEdit(owner)}
                    size="small"
                    sx={{ mr: 1 }}
                  >
                    <EditIcon />
                  </IconButton>
                  <IconButton
                    onClick={() => handleDeleteClick(owner)}
                    size="small"
                    color="error"
                  >
                    <DeleteIcon />
                  </IconButton>
                </>
              }
            >
              <ListItemText
                primary={owner.name}
                secondary={`创建时间: ${owner.created_at}`}
              />
            </ListItem>
          ))}
        </List>
      )}

      {/* 新增/编辑对话框 */}
      <Dialog
        open={dialogOpen}
        onClose={() => setDialogOpen(false)}
        maxWidth="xs"
        fullWidth
      >
        <DialogTitle>
          {dialogMode === "create" ? "新增归属人" : "编辑归属人"}
        </DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label="归属人名称"
            fullWidth
            variant="outlined"
            value={nameInput}
            onChange={(e) => setNameInput(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") handleSave();
            }}
            disabled={saving}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDialogOpen(false)} disabled={saving}>
            取消
          </Button>
          <Button
            onClick={handleSave}
            variant="contained"
            disabled={saving || !nameInput.trim()}
          >
            {saving ? <CircularProgress size={20} /> : "保存"}
          </Button>
        </DialogActions>
      </Dialog>

      {/* 删除确认对话框 */}
      <Dialog
        open={deleteDialogOpen}
        onClose={() => setDeleteDialogOpen(false)}
        maxWidth="xs"
        fullWidth
      >
        <DialogTitle>确认删除</DialogTitle>
        <DialogContent>
          <Typography>
            确定要删除归属人「{deletingOwner?.name}」吗？
          </Typography>
          <Typography color="error" variant="body2" sx={{ mt: 1 }}>
            如果该归属人已关联交易记录，删除后交易的 payer 字段将变为空。
          </Typography>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDeleteDialogOpen(false)} disabled={deleting}>
            取消
          </Button>
          <Button
            onClick={handleDeleteConfirm}
            color="error"
            variant="contained"
            disabled={deleting}
          >
            {deleting ? <CircularProgress size={20} /> : "删除"}
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}

export default AccountOwnersPage;
