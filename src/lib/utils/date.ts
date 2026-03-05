/** Format an ISO date string "YYYY-MM-DD" (or ISO datetime) to Jamaican "DD/MM/YYYY" */
export function formatDate(iso: string | null | undefined): string {
  if (!iso) return "—";
  const [y, m, d] = iso.split("T")[0].split("-");
  return `${d}/${m}/${y}`;
}
