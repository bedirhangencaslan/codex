/** @type {import('jest').Config} */
module.exports = {
  testEnvironment: "node",
  testMatch: ["<rootDir>/tests/**/*.test.ts", "<rootDir>/tests/**/*.test.tsx"],
  moduleNameMapper: {
    "^@protocol/(.*)$": "<rootDir>/../codex-rs/app-server-protocol/schema/typescript/$1",
  },
  transform: {
    "^.+\\.tsx?$": [
      "ts-jest",
      {
        tsconfig: {
          module: "commonjs",
          jsx: "react-jsx",
          esModuleInterop: true,
          resolveJsonModule: true,
          isolatedModules: true,
        },
      },
    ],
  },
};
