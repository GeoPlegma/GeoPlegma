const branch = process.env.GITHUB_REF_NAME || "dev"; // fallback to main if not set

export default {
  base: `/GeoPlegma/${branch}/`,
  title: "GeoPlegma",
};
