/**
 * OpenAI-compatible tool definitions voor de AI Configurator Assistant.
 * Deze tools worden meegegeven aan de API zodat het model function calls kan doen.
 */
export const AI_TOOLS = [
  {
    type: "function",
    function: {
      name: "create_kozijn",
      description: "Maak een nieuw kozijn aan vanuit een sjabloon. Gebruik dit als de gebruiker een nieuw raam, deur of kozijn wil.",
      parameters: {
        type: "object",
        properties: {
          template: {
            type: "string",
            enum: ["single_turn_tilt", "double_turn_tilt", "sliding_door", "front_door"],
            description: "Type kozijn: single_turn_tilt (draaikiep), double_turn_tilt (dubbel), sliding_door (schuifpui), front_door (voordeur)",
          },
          width: {
            type: "number",
            description: "Breedte in millimeters (bv. 900, 1200, 1800, 3000)",
          },
          height: {
            type: "number",
            description: "Hoogte in millimeters (bv. 1400, 1500, 2100, 2400)",
          },
          sjabloon: {
            type: "string",
            enum: ["standaard-67-meranti", "standaard-67-accoya", "zwaar-78-meranti", "passief-90-meranti"],
            description: "Profiel sjabloon: standaard-67-meranti (default), standaard-67-accoya, zwaar-78-meranti, passief-90-meranti",
          },
        },
        required: ["template", "width", "height"],
      },
    },
  },
  {
    type: "function",
    function: {
      name: "update_dimensions",
      description: "Wijzig de breedte en/of hoogte van het huidige kozijn.",
      parameters: {
        type: "object",
        properties: {
          width: { type: "number", description: "Nieuwe breedte in mm" },
          height: { type: "number", description: "Nieuwe hoogte in mm" },
        },
        required: ["width", "height"],
      },
    },
  },
  {
    type: "function",
    function: {
      name: "add_column",
      description: "Voeg een verticale verdeler (stijl/tussenstijl) toe op een positie in mm gemeten vanaf links.",
      parameters: {
        type: "object",
        properties: {
          position: { type: "number", description: "Positie in mm vanaf linkerkant" },
        },
        required: ["position"],
      },
    },
  },
  {
    type: "function",
    function: {
      name: "add_row",
      description: "Voeg een horizontale verdeler (dorpel/tussendorpel) toe op een positie in mm gemeten vanaf boven.",
      parameters: {
        type: "object",
        properties: {
          position: { type: "number", description: "Positie in mm vanaf bovenkant" },
        },
        required: ["position"],
      },
    },
  },
  {
    type: "function",
    function: {
      name: "set_cell_type",
      description: "Stel het type van een vak in (glas, draairaam, deur, paneel, etc.). Cell index 0 = linksboven, telt van links naar rechts, boven naar onder.",
      parameters: {
        type: "object",
        properties: {
          cell_index: { type: "integer", description: "Vak index (0-gebaseerd, links-naar-rechts, boven-naar-onder)" },
          panel_type: {
            type: "string",
            enum: ["fixed_glass", "turn_tilt", "turn", "tilt", "sliding", "door", "panel", "ventilation"],
            description: "Type paneel",
          },
          direction: {
            type: "string",
            enum: ["left", "right", "inward", "outward"],
            description: "Openingsrichting (optioneel, relevant voor draairamen en deuren)",
          },
        },
        required: ["cell_index", "panel_type"],
      },
    },
  },
  {
    type: "function",
    function: {
      name: "set_colors",
      description: "Stel de binnen- en buitenkleur van het kozijn in met RAL codes.",
      parameters: {
        type: "object",
        properties: {
          color_inside: { type: "string", description: "RAL kleurcode binnenzijde (bv. RAL9010)" },
          color_outside: { type: "string", description: "RAL kleurcode buitenzijde (bv. RAL7016)" },
        },
        required: ["color_inside", "color_outside"],
      },
    },
  },
  {
    type: "function",
    function: {
      name: "duplicate_kozijn",
      description: "Dupliceer het huidige kozijn met een nieuw merkteken.",
      parameters: {
        type: "object",
        properties: {
          new_mark: { type: "string", description: "Nieuw merkteken (bv. K02, K03)" },
        },
        required: ["new_mark"],
      },
    },
  },
  {
    type: "function",
    function: {
      name: "calculate_thermal",
      description: "Bereken de thermische waarde (Uw) van het huidige kozijn. Geen parameters nodig.",
      parameters: { type: "object", properties: {} },
    },
  },
  {
    type: "function",
    function: {
      name: "get_current_kozijn",
      description: "Haal informatie op over het huidige kozijn (afmetingen, type, vakken, kleuren, profielen). Gebruik dit om de gebruiker te informeren.",
      parameters: { type: "object", properties: {} },
    },
  },
];
