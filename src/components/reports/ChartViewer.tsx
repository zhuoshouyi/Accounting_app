import { Component } from "react";
import { Box, Typography, Alert } from "@mui/material";
import { BarChart, Bar, LineChart, Line, PieChart, Pie, AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, Legend, Cell } from "recharts";

const COLORS = ["#4caf50","#ff9800","#2196f3","#e91e63","#9c27b0","#00bcd4","#ff5722","#607d8b","#795548","#cddc39","#3f51b5","#009688","#ff4081","#03a9f4","#8bc34a"];

class ErrorBoundary extends Component<{ children: React.ReactNode }, { hasError: boolean; err: string }> {
  constructor(p: any) { super(p); this.state = { hasError: false, err: "" }; }
  static getDerivedStateFromError(e: Error) { return { hasError: true, err: e.message }; }
  render() {
    if (this.state.hasError) return <Alert severity="error">图表渲染出错: {this.state.err}</Alert>;
    return this.props.children;
  }
}

function ChartViewer({ config }: { config: any }) {
  if (!config || !config.data || !config.data.length) return <Typography color="text.secondary">无数据</Typography>;

  const data = config.data;
  const xKey = config.xKey || "name";
  const yKey = config.yKey || "value";
  const type = config.type || "bar";
  const groupKey = config.groupKey;

  const renderChart = () => {
    if (type === "pie") {
      return (
        <PieChart>
          <Pie data={data} dataKey={yKey} nameKey={xKey} cx="50%" cy="50%" outerRadius={130} label>
            {data.map((_: any, i: number) => <Cell key={i} fill={COLORS[i % COLORS.length]} />)}
          </Pie>
          <Tooltip /><Legend />
        </PieChart>
      );
    }

    const groups: string[] = groupKey ? [...new Set((data as any[]).map((d: any) => String(d[groupKey] || "")))] as string[] : [];
    const wrapperMap: any = { bar: BarChart, line: LineChart, area: AreaChart };
    const shapeMap: any = { bar: Bar, line: Line, area: Area };
    const Wrapper = (wrapperMap as any)[type] || BarChart;
    const Shape = (shapeMap as any)[type] || Bar;

    return (
      <Wrapper data={data}>
        <CartesianGrid strokeDasharray="3 3" />
        <XAxis dataKey={xKey} />
        <YAxis />
        <Tooltip /><Legend />
        {groups.length > 0
          ? groups.map((g: any, i: number) => <Shape key={String(g)} dataKey={yKey} data={(data as any[]).filter((d: any) => String(d[groupKey] || "") === g)} name={String(g)} fill={COLORS[i % COLORS.length]} />)
          : <Shape dataKey={yKey} fill={COLORS[0]} />}
      </Wrapper>
    );
  };

  return (
    <Box sx={{ width: "100%", height: 380 }}>
      <ErrorBoundary>
        {renderChart()}
      </ErrorBoundary>
    </Box>
  );
}

export default ChartViewer;
