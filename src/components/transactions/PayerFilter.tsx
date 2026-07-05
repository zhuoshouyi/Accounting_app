import { forwardRef, useImperativeHandle, useState } from "react";
import { Select, MenuItem, FormControl } from "@mui/material";

/** 归属人浮动筛选器（AG Grid floating filter） */
const PayerFilter = forwardRef((props: any, ref) => {
  const [value, setValue] = useState<string>("");

  useImperativeHandle(ref, () => ({
    onParentModelChanged(parentModel: any) {
      const v = typeof parentModel === "string" ? parentModel : (parentModel?.filter || parentModel || "");
      setValue(v);
    },
  }));

  const owners: string[] = props.filterParams?.values || [];

  const handleChange = (v: string) => {
    setValue(v);
    if (v) {
      props.parentFilterInstance?.((inst: any) => {
        inst.onFloatingFilterChanged("equals", v);
      });
    } else {
      props.parentFilterInstance?.((inst: any) => {
        inst.onFloatingFilterChanged(null, null);
      });
    }
  };

  return (
    <FormControl size="small" fullWidth sx={{ minWidth: 80 }}>
      <Select value={value} displayEmpty onChange={(e) => handleChange(e.target.value)}
        sx={{ fontSize: 12, "& .MuiSelect-select": { py: 0.5 } }}>
        <MenuItem value="">全部</MenuItem>
        {owners.filter(Boolean).map((name: string) => (
          <MenuItem key={name} value={name}>{name}</MenuItem>
        ))}
      </Select>
    </FormControl>
  );
});

PayerFilter.displayName = "PayerFilter";
export default PayerFilter;
