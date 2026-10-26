import "./Certificates.css";
import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useRef, useState } from "react";
import { toast } from "sonner";
import { useError } from "../ErrorContext";
import type { AppError } from "../errors";
import { useTranslation } from "react-i18next";

type ProfileInfo = {
  uuid: string;
  name: string;
  appIdName: string | null;
  appIdentifier: string | null;
  teamName: string | null;
  created: string | null;
  expires: string;
};

const HOUR = 60 * 60 * 1000;

const statusColor = (hoursLeft: number) => {
  if (hoursLeft <= 0) return "#e5484d";
  if (hoursLeft <= 24) return "#f76b15";
  if (hoursLeft <= 72) return "#ffc53d";
  return "#30a46c";
};

export const Expiry = () => {
  const { t } = useTranslation();
  const [profiles, setProfiles] = useState<ProfileInfo[]>([]);
  const [loading, setLoading] = useState<boolean>(false);
  const loadingRef = useRef<boolean>(false);

  const { err } = useError();

  const loadProfiles = useCallback(async () => {
    if (loadingRef.current) return;
    const promise = async () => {
      loadingRef.current = true;
      setLoading(true);
      try {
        setProfiles(await invoke<ProfileInfo[]>("list_installed_profiles"));
      } finally {
        setLoading(false);
        loadingRef.current = false;
      }
    };
    toast.promise(promise, {
      loading: t("expiry.loading"),
      success: t("expiry.loaded_success"),
      error: (e) => err(t("expiry.failed_load"), e),
    });
  }, [t]);

  const exportCalendar = useCallback(async () => {
    try {
      const count = await invoke<number | null>("export_expiry_calendar", {
        dialogTitle: t("expiry.export_calendar"),
        filterName: t("expiry.calendar_files"),
        titleTemplate: t("expiry.event_title", { name: "{name}" }),
        description: t("expiry.event_description"),
      });
      if (count !== null) toast.success(t("expiry.exported_success", { count }));
    } catch (e) {
      toast.error(err(t("expiry.failed_export"), e as AppError));
    }
  }, [t]);

  useEffect(() => {
    loadProfiles();
  }, []);

  const now = Date.now();

  return (
    <>
      <h2>{t("expiry.manage")}</h2>
      <div style={{ marginBottom: "0.5em" }}>{t("expiry.explanation")}</div>
      {profiles.length === 0 ? (
        <div>{loading ? t("expiry.loading") : t("expiry.none_found")}</div>
      ) : (
        <div className="card">
          <div className="certificate-table-container">
            <table className="certificate-table">
              <thead>
                <tr className="certificate-item">
                  <th className="cert-item-part">{t("expiry.name")}</th>
                  <th className="cert-item-part">{t("expiry.expires")}</th>
                  <th className="cert-item-part" style={{ borderRight: "none" }}>
                    {t("expiry.time_left")}
                  </th>
                </tr>
              </thead>
              <tbody>
                {profiles.map((profile, i) => {
                  const hoursLeft = (Date.parse(profile.expires) - now) / HOUR;
                  return (
                    <tr
                      key={profile.uuid}
                      className={
                        "certificate-item" +
                        (i === profiles.length - 1 ? " cert-item-last" : "")
                      }
                    >
                      <td className="cert-item-part" title={profile.appIdentifier ?? ""}>
                        {profile.appIdName ?? profile.name}
                      </td>
                      <td className="cert-item-part">
                        {new Date(profile.expires).toLocaleString()}
                      </td>
                      <td
                        className="cert-item-part"
                        style={{ borderRight: "none", color: statusColor(hoursLeft) }}
                      >
                        {hoursLeft <= 0
                          ? t("expiry.expired")
                          : hoursLeft < 24
                            ? t("expiry.hours_left", { count: Math.floor(hoursLeft) })
                            : t("expiry.days_left", { count: Math.floor(hoursLeft / 24) })}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>
      )}
      <div style={{ display: "flex", gap: "0.5em", marginTop: "1em" }}>
        <button style={{ flex: 1 }} onClick={loadProfiles} disabled={loading}>
          {t("common.refresh")}
        </button>
        <button
          style={{ flex: 1 }}
          onClick={exportCalendar}
          disabled={loading || profiles.length === 0}
        >
          {t("expiry.export_calendar")}
        </button>
      </div>
    </>
  );
};
