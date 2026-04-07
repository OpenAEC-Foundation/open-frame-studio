/**
 * AI Configurator Assistant store.
 * Beheert chat state, API communicatie en tool call execution.
 */
import { writable, get } from "svelte/store";
import { getSetting, saveSetting } from "../lib/settings.js";
import { SYSTEM_PROMPT } from "../lib/aiSystemPrompt.js";
import { AI_TOOLS } from "../lib/aiTools.js";
import {
  createFromTemplate,
  updateDimensions,
  addColumn,
  addRow,
  updateCellType,
  updateFrameColors,
  duplicateKozijn,
  calculateThermal,
  currentKozijn,
} from "./kozijn.js";
import { toast } from "./toast.js";

export const aiMessages = writable([]);
export const aiLoading = writable(false);
export const aiConfigured = writable(false);

// Load saved config
const savedEndpoint = getSetting("ai_endpoint") || "";
const savedModel = getSetting("ai_model") || "gpt-4o-mini";
const savedKey = getSetting("ai_key") || "";

export const aiEndpoint = writable(savedEndpoint);
export const aiModel = writable(savedModel);
export const aiKey = writable(savedKey);

// Check if configured
if (savedEndpoint) aiConfigured.set(true);

export function configureAi(endpoint, key, model) {
  aiEndpoint.set(endpoint);
  aiKey.set(key);
  aiModel.set(model);
  saveSetting("ai_endpoint", endpoint);
  saveSetting("ai_model", model);
  saveSetting("ai_key", key);
  aiConfigured.set(!!endpoint);
}

export function clearChat() {
  aiMessages.set([]);
}

export async function sendMessage(text) {
  if (!text.trim()) return;

  const endpoint = get(aiEndpoint);
  const key = get(aiKey);
  const model = get(aiModel);

  if (!endpoint) {
    toast.warning("Configureer eerst de AI API instellingen.");
    return;
  }

  // Add user message
  aiMessages.update((msgs) => [...msgs, { role: "user", content: text }]);
  aiLoading.set(true);

  try {
    // Build messages array for API
    const msgs = get(aiMessages);
    const apiMessages = [
      { role: "system", content: SYSTEM_PROMPT },
      ...msgs.map((m) => ({
        role: m.role,
        content: m.content,
      })),
    ];

    // Call OpenAI-compatible API
    const response = await fetch(`${endpoint}/chat/completions`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...(key ? { Authorization: `Bearer ${key}` } : {}),
      },
      body: JSON.stringify({
        model,
        messages: apiMessages,
        tools: AI_TOOLS,
        tool_choice: "auto",
        temperature: 0.3,
      }),
    });

    if (!response.ok) {
      const errText = await response.text();
      throw new Error(`API fout ${response.status}: ${errText}`);
    }

    const data = await response.json();
    const choice = data.choices?.[0];

    if (!choice) {
      throw new Error("Geen antwoord van de AI.");
    }

    const message = choice.message;

    // Process tool calls if present
    if (message.tool_calls && message.tool_calls.length > 0) {
      const toolResults = [];

      for (const toolCall of message.tool_calls) {
        const name = toolCall.function.name;
        const args = JSON.parse(toolCall.function.arguments || "{}");

        try {
          const result = await executeToolCall(name, args);
          toolResults.push({
            name,
            args,
            success: true,
            result: typeof result === "object" ? summarizeResult(name, result) : String(result),
          });
        } catch (e) {
          toolResults.push({
            name,
            args,
            success: false,
            result: String(e),
          });
        }
      }

      // Add assistant message with tool results
      const toolSummary = toolResults
        .map((t) => (t.success ? `✓ ${t.name}: ${t.result}` : `✗ ${t.name}: ${t.result}`))
        .join("\n");

      const contentText = message.content || "";
      aiMessages.update((msgs) => [
        ...msgs,
        {
          role: "assistant",
          content: contentText,
          toolCalls: toolResults,
        },
      ]);

      // If there was content from the model too, and tool calls need a follow-up,
      // send tool results back for a natural language response
      if (toolResults.length > 0) {
        const followUp = await getFollowUpResponse(
          [...get(aiMessages)],
          message.tool_calls,
          toolResults
        );
        if (followUp && followUp !== contentText) {
          aiMessages.update((msgs) => {
            const last = msgs[msgs.length - 1];
            if (last.role === "assistant") {
              return [...msgs.slice(0, -1), { ...last, content: followUp }];
            }
            return [...msgs, { role: "assistant", content: followUp }];
          });
        }
      }
    } else {
      // Plain text response
      aiMessages.update((msgs) => [
        ...msgs,
        { role: "assistant", content: message.content || "..." },
      ]);
    }
  } catch (e) {
    console.error("AI fout:", e);
    aiMessages.update((msgs) => [
      ...msgs,
      { role: "assistant", content: `Fout: ${e.message}`, isError: true },
    ]);
  } finally {
    aiLoading.set(false);
  }
}

async function getFollowUpResponse(messages, toolCalls, toolResults) {
  const endpoint = get(aiEndpoint);
  const key = get(aiKey);
  const model = get(aiModel);

  try {
    // Build tool response messages
    const toolResponseMsgs = toolCalls.map((tc, i) => ({
      role: "tool",
      tool_call_id: tc.id,
      content: JSON.stringify(toolResults[i]),
    }));

    const apiMessages = [
      { role: "system", content: SYSTEM_PROMPT },
      ...messages.map((m) => ({ role: m.role, content: m.content || "" })),
      ...toolResponseMsgs,
    ];

    const response = await fetch(`${endpoint}/chat/completions`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        ...(key ? { Authorization: `Bearer ${key}` } : {}),
      },
      body: JSON.stringify({
        model,
        messages: apiMessages,
        temperature: 0.3,
      }),
    });

    if (!response.ok) return null;
    const data = await response.json();
    return data.choices?.[0]?.message?.content || null;
  } catch {
    return null;
  }
}

/**
 * Execute a tool call by name, routing to the correct store function.
 */
async function executeToolCall(name, args) {
  switch (name) {
    case "create_kozijn": {
      const { template, width, height, sjabloon } = args;
      const k = await createFromTemplate(
        template || "single_turn_tilt",
        width || 900,
        height || 1400,
        sjabloon || "standaard-67-meranti"
      );
      toast.success(`Kozijn aangemaakt: ${width}×${height}mm`);
      return k;
    }

    case "update_dimensions": {
      const { width, height } = args;
      await updateDimensions(width, height);
      toast.success(`Afmetingen gewijzigd: ${width}×${height}mm`);
      return { width, height };
    }

    case "add_column": {
      const { position } = args;
      await addColumn(position);
      toast.success(`Kolom toegevoegd op ${position}mm`);
      return { position };
    }

    case "add_row": {
      const { position } = args;
      await addRow(position);
      toast.success(`Rij toegevoegd op ${position}mm`);
      return { position };
    }

    case "set_cell_type": {
      const { cell_index, panel_type, direction } = args;
      await updateCellType(cell_index, panel_type, direction || null);
      toast.success(`Vak ${cell_index} ingesteld als ${panel_type}`);
      return { cell_index, panel_type, direction };
    }

    case "set_colors": {
      const { color_inside, color_outside } = args;
      await updateFrameColors(color_inside, color_outside);
      toast.success(`Kleuren: binnen ${color_inside}, buiten ${color_outside}`);
      return { color_inside, color_outside };
    }

    case "duplicate_kozijn": {
      const { new_mark } = args;
      const dup = await duplicateKozijn(new_mark);
      toast.success(`Kozijn gedupliceerd als ${new_mark}`);
      return dup;
    }

    case "calculate_thermal": {
      const result = await calculateThermal();
      if (result) {
        toast.info(`Uw-waarde: ${result.uwValue} W/m²K (${result.rating})`);
      }
      return result;
    }

    case "get_current_kozijn": {
      const k = get(currentKozijn);
      if (!k) return { error: "Geen kozijn geselecteerd" };
      return {
        name: k.name,
        mark: k.mark,
        width: k.frame?.outerWidth,
        height: k.frame?.outerHeight,
        material: k.frame?.material,
        colorInside: k.frame?.colorInside,
        colorOutside: k.frame?.colorOutside,
        columns: k.grid?.columns?.length,
        rows: k.grid?.rows?.length,
        cells: k.cells?.map((c, i) => ({
          index: i,
          type: c.panelType,
          direction: c.openingDirection,
        })),
      };
    }

    default:
      throw new Error(`Onbekende tool: ${name}`);
  }
}

/**
 * Summarize a tool call result for display in chat.
 */
function summarizeResult(name, result) {
  if (!result) return "OK";
  switch (name) {
    case "create_kozijn":
      return `Kozijn ${result.mark || ""} (${result.frame?.outerWidth}×${result.frame?.outerHeight}mm)`;
    case "update_dimensions":
      return `${result.width}×${result.height}mm`;
    case "calculate_thermal":
      return result.uwValue ? `Uw = ${result.uwValue} W/m²K` : "Berekend";
    case "get_current_kozijn":
      return result.error || `${result.name} (${result.width}×${result.height}mm, ${result.cells?.length} vakken)`;
    default:
      return "OK";
  }
}
