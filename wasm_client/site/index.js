/* global uPlot */
import init, { start_log_stream } from "../pkg/wasm_client.js";

async function run() {
  await init();
  start_log_stream();

  let logTimestamps = [];
  let metricTimestamps = [];

  // --- 1. Log Traffic Chart ---
  const logOpts = {
    title: "Log Traffic (IST)",
    width: 600,
    height: 250,
    scales: { x: { time: false } },
    series: [
      {},
      { label: "Total Logs", stroke: "#6366f1", width: 2 },
      {
        label: "Errors",
        stroke: "#ef4444",
        width: 2,
        fill: "rgba(239, 68, 68, 0.1)",
      },
    ],
    axes: [
      {
        stroke: "#9ca3af",
        grid: { stroke: "#374151" },
        // Map the index (0,1,2...) to the actual HH:mm:ss string
        values: (self, ticks) => ticks.map((idx) => logTimestamps[idx] || ""),
      },
      { stroke: "#9ca3af", grid: { stroke: "#374151" } },
    ],
  };

  const logUplot = new uPlot(
    logOpts,
    [[], [], []],
    document.getElementById("chart-container"),
  );

  // --- 2. System Metrics Chart (CPU & RAM) ---
  const metricOpts = {
    title: "System Performance (%)",
    width: 600,
    height: 250,
    scales: { x: { time: false }, y: { range: [0, 100] } },
    series: [
      {},
      { label: "CPU Usage", stroke: "#3b82f6", width: 2 },
      { label: "RAM Usage", stroke: "#10b981", width: 2 },
    ],
    axes: [
      {
        stroke: "#9ca3af",
        grid: { stroke: "#374151" },
        // Map the index to the actual HH:mm:ss string
        values: (self, ticks) =>
          ticks.map((idx) => metricTimestamps[idx] || ""),
      },
      {
        stroke: "#9ca3af",
        grid: { stroke: "#374151" },
        values: (self, ticks) => ticks.map((t) => t + "%"),
      },
    ],
  };

  const metricUplot = new uPlot(
    metricOpts,
    [[], [], []],
    document.getElementById("metric-container"),
  );

  window.updateLogUPlot = (x, yTotal, yErrors, timestamps) => {
    logTimestamps = timestamps;
    logUplot.setData([x, yTotal, yErrors]);
  };

  window.updateMetricUPlot = (x, cpu, ram, timestamps) => {
    metricTimestamps = timestamps;
    metricUplot.setData([x, cpu, ram]);
  };
}

run();
