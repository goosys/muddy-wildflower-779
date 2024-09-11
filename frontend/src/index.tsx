import React from "react";
import ReactDOM from "react-dom/client";
import { HaikunatorGenerator } from "./pages/HaikunatorGenerator";

import "./index.css";

const root = document.getElementById("root");

if (!root) {
  throw new Error("No root element found");
}

ReactDOM.createRoot(root).render(
  <React.StrictMode>
    <HaikunatorGenerator />
  </React.StrictMode>,
);
