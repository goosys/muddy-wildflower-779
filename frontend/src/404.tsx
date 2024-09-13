import React from "react";
import ReactDOM from "react-dom/client";

import "./index.css";

const root = document.getElementById("root");

if (!root) {
  throw new Error("No root element found");
}

ReactDOM.createRoot(root).render(
  <React.StrictMode>
    <div className="flex items-center justify-center h-screen">
      <h1 className="text-4xl font-bold text-gray-500">404 Not Found</h1>
    </div>
  </React.StrictMode>
);
