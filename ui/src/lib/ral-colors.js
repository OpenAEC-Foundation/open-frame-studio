/**
 * RAL kleuren database — meestgebruikte kleuren in de kozijnindustrie.
 */
export const RAL_COLORS = [
  { code: "RAL9010", name: "Zuiver wit", hex: "#F1ECE1" },
  { code: "RAL9001", name: "Crèmewit", hex: "#F0EDE3" },
  { code: "RAL9016", name: "Verkeerswit", hex: "#F7F9EF" },
  { code: "RAL7016", name: "Antracietgrijs", hex: "#383E42" },
  { code: "RAL9005", name: "Gitzwart", hex: "#0E0E10" },
  { code: "RAL7021", name: "Zwartgrijs", hex: "#2F3234" },
  { code: "RAL7035", name: "Lichtgrijs", hex: "#C5C7C4" },
  { code: "RAL7039", name: "Kwartsgrijs", hex: "#6B6B60" },
  { code: "RAL8014", name: "Sepiabruin", hex: "#49392D" },
  { code: "RAL8003", name: "Leembruin", hex: "#7E4B26" },
  { code: "RAL6009", name: "Dennegroen", hex: "#1F3A28" },
  { code: "RAL6005", name: "Mosgroen", hex: "#1E3B2B" },
  { code: "RAL5011", name: "Staalblauw", hex: "#1A2B3C" },
  { code: "RAL3005", name: "Wijnrood", hex: "#5E2028" },
  { code: "RAL1015", name: "Licht ivoor", hex: "#E3D4B5" },
  { code: "RAL7022", name: "Ombergrijs", hex: "#4B4D46" },
  { code: "RAL9007", name: "Grijs aluminium", hex: "#8C8C7E" },
  { code: "RAL9006", name: "Blank aluminium", hex: "#A1A1A0" },
  { code: "RAL8022", name: "Zwartbruin", hex: "#1A1718" },
  { code: "RAL7015", name: "Leigrijs", hex: "#4D5258" },
];

export function ralToHex(code) {
  const color = RAL_COLORS.find(c => c.code === code);
  return color ? color.hex : "#CCCCCC";
}
