// Goal item 9: API keys and prices. Keys go to VS Code's SecretStorage and reach Suffice as
// environment variables of the app-server the extension starts (a custom provider's env_key,
// e.g. ZAI_API_KEY); an OpenAI key goes through account/login/start. Prices feed the chat's cost
// meter: built-in figures plus the user's own, per model.
import type { GetAccountResponse } from "@protocol/v2/GetAccountResponse";
import { useEffect, useState } from "react";
import { BUILTIN_PRICES, costOf, parsePrice, priceFor } from "../../shared/pricing";
import { formatUsd } from "../../shared/meters";
import { useApp } from "../app/context";
import { Icon } from "../components/icons";
import { Badge, Button, Notice, Section } from "../components/ui";

export function ApiScreen() {
  const { t } = useApp();
  return (
    <div className="sf-screen">
      <header className="sf-screen-head">
        <h1>{t("api.title")}</h1>
        <p className="sf-muted">{t("api.intro")}</p>
      </header>
      <EnvKeys />
      <Account />
      <Prices />
    </div>
  );
}

function EnvKeys() {
  const { state, ctl, t } = useApp();
  const [values, setValues] = useState<Record<string, string>>({});
  const [newName, setNewName] = useState("");
  const keys = state.init?.envKeys ?? [];
  return (
    <Section title={t("api.providersTitle")}>
      <ul className="sf-key-list">
        {keys.map((k) => (
          <li key={k.name} className="sf-key">
            <div className="sf-key-head">
              <Icon name="key" size={13} />
              <code>{k.name}</code>
              <Badge tone={k.stored ? "success" : "neutral"}>{k.stored ? t("api.keyStored") : t("api.keyMissing")}</Badge>
            </div>
            <form
              className="sf-row"
              onSubmit={(e) => {
                e.preventDefault();
                const v = values[k.name]?.trim();
                if (v) {
                  ctl.saveKey(k.name, v);
                  setValues({ ...values, [k.name]: "" });
                }
              }}
            >
              <input
                className="sf-input"
                type="password"
                autoComplete="off"
                value={values[k.name] ?? ""}
                onChange={(e) => setValues({ ...values, [k.name]: e.target.value })}
                placeholder={t("api.keyPlaceholder")}
                aria-label={`${k.name} ${t("api.keyPlaceholder")}`}
              />
              <Button variant="primary" type="submit" disabled={!values[k.name]?.trim()}>
                {t("api.saveKey")}
              </Button>
              {k.stored && (
                <Button variant="ghost" onClick={() => ctl.saveKey(k.name, null)}>
                  {t("api.removeKey")}
                </Button>
              )}
            </form>
          </li>
        ))}
      </ul>
      <p className="sf-muted">{t("api.restartNotice")}</p>
      <form
        className="sf-row"
        onSubmit={(e) => {
          e.preventDefault();
          const name = newName.trim().toUpperCase();
          const value = values[`new:${name}`]?.trim();
          if (/^[A-Z_][A-Z0-9_]*$/.test(name) && value) {
            ctl.saveKey(name, value);
            setNewName("");
          }
        }}
      >
        <input className="sf-input" value={newName} onChange={(e) => setNewName(e.target.value)} placeholder={t("api.addProviderName")} aria-label={t("api.addProviderName")} />
        <input
          className="sf-input"
          type="password"
          value={values[`new:${newName.trim().toUpperCase()}`] ?? ""}
          onChange={(e) => setValues({ ...values, [`new:${newName.trim().toUpperCase()}`]: e.target.value })}
          placeholder={t("api.keyPlaceholder")}
          aria-label={t("api.keyPlaceholder")}
        />
        <Button type="submit" icon="plus">
          {t("api.addProvider")}
        </Button>
      </form>
    </Section>
  );
}

function Account() {
  const { state, ctl, t } = useApp();
  const [account, setAccount] = useState<GetAccountResponse | null>(null);
  const [key, setKey] = useState("");
  useEffect(() => {
    if (state.server.state === "ready") void ctl.session.accountRead({ refreshToken: false }).then(setAccount).catch(() => setAccount(null));
  }, [state.server.state]);
  if (!account) return null;
  const a = account.account;
  return (
    <Section title={t("api.accountTitle")}>
      {!account.requiresOpenaiAuth ? (
        <Notice tone="info">{t("api.accountNotRequired")}</Notice>
      ) : (
        <p>{a?.type === "apiKey" ? t("api.accountApiKey") : a?.type === "chatgpt" ? t("api.accountChatgpt", { email: a.email ?? "–" }) : t("api.accountNone")}</p>
      )}
      <form
        className="sf-row"
        onSubmit={(e) => {
          e.preventDefault();
          if (key.trim()) void ctl.loginWithOpenAiKey(key.trim()).then(() => setKey(""));
        }}
      >
        <input className="sf-input" type="password" value={key} onChange={(e) => setKey(e.target.value)} placeholder="sk-…" aria-label={t("api.openaiKeyLogin")} />
        <Button type="submit" disabled={!key.trim()}>
          {t("api.openaiKeyLogin")}
        </Button>
      </form>
    </Section>
  );
}

function Prices() {
  const { state, ctl, t } = useApp();
  const overrides = state.init?.prices ?? {};
  const models = [...new Set([...state.models.map((m) => m.id), ...Object.keys(BUILTIN_PRICES), ...Object.keys(overrides)])];
  const [editing, setEditing] = useState<string | null>(null);
  const [draft, setDraft] = useState({ input: "", cached: "", output: "" });
  const [error, setError] = useState(false);
  const example = BUILTIN_PRICES["glm-5.3-flash"]!;
  const exampleCost = costOf({ inputTokens: 10_000, cachedInputTokens: 8_000, outputTokens: 500 }, priceFor("glm-5.3-flash", overrides) ?? example);
  return (
    <Section title={t("api.pricingTitle")} description={t("api.pricingIntro")}>
      <div className="sf-table-wrap">
        <table className="sf-table">
          <thead>
            <tr>
              <th>{t("api.colModel")}</th>
              <th>{t("api.colInput")}</th>
              <th>{t("api.colCached")}</th>
              <th>{t("api.colOutput")}</th>
              <th>{t("api.colSource")}</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {models.map((model) => {
              const price = priceFor(model, overrides);
              if (editing === model) {
                return (
                  <tr key={model}>
                    <td>{model}</td>
                    {(["input", "cached", "output"] as const).map((f) => (
                      <td key={f}>
                        <input className="sf-input sf-input-num" inputMode="decimal" value={draft[f]} onChange={(e) => setDraft({ ...draft, [f]: e.target.value })} aria-label={`${model} ${f}`} />
                      </td>
                    ))}
                    <td colSpan={2}>
                      <Button
                        variant="primary"
                        onClick={() => {
                          const parsed = parsePrice(draft);
                          if (!parsed) return setError(true);
                          ctl.setPrice(model, parsed);
                          setEditing(null);
                          setError(false);
                        }}
                      >
                        {t("common.save")}
                      </Button>
                      <Button variant="ghost" onClick={() => setEditing(null)}>
                        {t("common.cancel")}
                      </Button>
                    </td>
                  </tr>
                );
              }
              return (
                <tr key={model}>
                  <td title={price?.note}>{model}</td>
                  {price ? (
                    <>
                      <td>${price.inputPerMillion}</td>
                      <td>${price.cachedInputPerMillion}</td>
                      <td>${price.outputPerMillion}</td>
                      <td>
                        <Badge tone={price.source === "user" ? "accent" : "neutral"}>{price.source === "user" ? t("api.sourceUser") : t("api.sourceBuiltin")}</Badge>
                      </td>
                    </>
                  ) : (
                    <td colSpan={4} className="sf-muted">
                      {t("api.noPrice")}
                    </td>
                  )}
                  <td className="sf-nowrap">
                    <Button
                      variant="ghost"
                      icon="pencil"
                      aria-label={`${t("common.edit")} ${model}`}
                      onClick={() => {
                        setDraft({ input: String(price?.inputPerMillion ?? ""), cached: String(price?.cachedInputPerMillion ?? ""), output: String(price?.outputPerMillion ?? "") });
                        setEditing(model);
                        setError(false);
                      }}
                    />
                    {overrides[model] && <Button variant="ghost" icon="refresh" aria-label={`${t("common.reset")} ${model}`} onClick={() => ctl.setPrice(model, null)} />}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
      {error && <Notice tone="danger">{t("api.invalidPrice")}</Notice>}
      <p className="sf-muted">{t("api.exampleCost", { cost: formatUsd(exampleCost.total) })}</p>
    </Section>
  );
}
