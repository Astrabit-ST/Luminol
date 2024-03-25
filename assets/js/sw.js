//! This is a slightly modified version of coi-serviceworker, commit 7b1d2a092d0d2dd2b7270b6f12f13605de26f214
//! https://github.com/gzuidhof/coi-serviceworker
/*!
 * MIT License
 *
 * Copyright (c) 2021 Guido Zuidhof
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
*/

const CACHE_NAME = "astrabit.luminol";

let coepCredentialless = true;
if (typeof window === 'undefined') {
    self.addEventListener("install", () => self.skipWaiting());
    self.addEventListener("activate", (event) => event.waitUntil(self.clients.claim()));

    self.addEventListener("message", (ev) => {
        if (!ev.data) {
            return;
        } else if (ev.data.type === "deregister") {
            self.registration
                .unregister()
                .then(() => {
                    return self.clients.matchAll();
                })
                .then(clients => {
                    clients.forEach((client) => client.navigate(client.url));
                });
        } else if (ev.data.type === "coepCredentialless") {
            coepCredentialless = ev.data.value;
        }
    });

    self.addEventListener("fetch", function (event) {
        const r = event.request;
        if (r.cache === "only-if-cached" && r.mode !== "same-origin") {
            return;
        }

        const request = (coepCredentialless && r.mode === "no-cors")
            ? new Request(r, {
                credentials: "omit",
            })
            : new URL(r.url).searchParams.has("luminol-invalidator")
            ? (() => {
                // Remove 'luminol-invalidator' from the request's query string if it exists in the query string
                const url = new URL(r.url);
                url.searchParams.delete("luminol-invalidator");
                return new Request(url, r);
            })()
            : r;

        const url = new URL(request.url);
        url.hash = "";
        url.pathname = url.pathname.trim();
        // Unescape escape codes like "%2f"
        url.pathname = decodeURIComponent(url.pathname);
        // Replace backslashes with forward slashes
        url.pathname = url.pathname.replace(/\\/g, "/");
        // Collapse repeated slashes
        url.pathname = url.pathname.replace(/\/+/g, "/");
        // Remove trailing slashes
        if (url.pathname.endsWith("/")) url.pathname = url.pathname.slice(0, -1);
        // Strip "index.html" from the end
        if (url.pathname.endsWith("/index.html")) url.pathname = url.pathname.slice(0, -11);

        event.respondWith(
            self.caches
                .match(url)
                .then((cached) => cached || fetch(request)) // Respond with cached response if one exists for this request
                .then((response) => {
                    if (response.status === 0) {
                        return new Response();
                    }

                    const newHeaders = new Headers(response.headers);
                    newHeaders.set("Cross-Origin-Embedder-Policy",
                        coepCredentialless ? "credentialless" : "require-corp"
                    );
                    if (!coepCredentialless) {
                        newHeaders.set("Cross-Origin-Resource-Policy", "cross-origin");
                    }
                    newHeaders.set("Cross-Origin-Opener-Policy", "same-origin");

                    const newResponse = new Response(response.body, {
                        status: response.status,
                        statusText: response.statusText,
                        headers: newHeaders,
                    });

                    // Auto-cache non-error, non-opaque responses for all same-origin requests other than buildinfo.json
                    if (response.type === "error" || url.origin !== self.origin || url.pathname.endsWith("/buildinfo.json")) {
                        return newResponse;
                    } else {
                        return self.caches
                            .open(CACHE_NAME)
                            .then((cache) => cache.put(url, newResponse.clone()))
                            .then(() => newResponse);
                    }
                })
                .catch((e) => {
                    if (!url.pathname.endsWith("/buildinfo.json")) {
                        console.error(e);
                    }
                })
        );
    });

} else {
    (() => {
        // Check for the current Luminol build info, and then clear the cache storage if
        // it doesn't match the build info we previously stored in local storage
        if (!window.sessionStorage.getItem("luminolCheckedForUpdate") && window.sessionStorage.getItem("coiReloadedAfterSuccess")) {
            (
                window.location.hash === "#dev"
                    ? Promise.resolve(null)
                    : fetch("./buildinfo.json")
                        .then((response) => {
                            if (response.status === 200) {
                                return response.json();
                            } else {
                                console.warn("Error checking for Luminol updates: request returned status code", response.status);
                            }
                        })
            )
                .then((info) => {
                    if (info === undefined) {
                        return;
                    }
                    const oldInfo = JSON.parse(window.localStorage.getItem("luminolBuildInfo"));
                    if (
                        info === null
                            || oldInfo === null
                            || info.epoch !== oldInfo.epoch
                            || info.rev !== oldInfo.rev
                            || info.profile !== oldInfo.profile
                            || info.profile !== "release"
                            || info.rev.endsWith("-modified")
                    ) {
                        !coi.quiet && console.log("New Luminol update detected - clearing cache.");
                        return window.caches.delete(CACHE_NAME).then(() => info);
                    }
                })
                .then((info) => {
                    if (info === undefined) {
                        return false;
                    }
                    window.sessionStorage.clear();
                    window.localStorage.setItem("luminolBuildInfo", JSON.stringify(info));
                    window.sessionStorage.setItem("luminolCheckedForUpdate", "true");
                    return window.navigator?.serviceWorker.getRegistration()
                        .then((registration) => registration?.unregister())
                        .then(() => true)
                            ?? true;
                })
                .then((shouldRefresh) => {
                    if (!shouldRefresh) {
                        return;
                    }
                    !coi.quiet && console.log("Reloading page to finish clearing cache.");
                    coi.doReload();
                })
                .catch((e) => console.warn("Error checking for Luminol updates:", e));
        }

        const reloadedBySelf = window.sessionStorage.getItem("coiReloadedBySelf");
        window.sessionStorage.removeItem("coiReloadedBySelf");
        const coepDegrading = (reloadedBySelf == "coepdegrade");

        // You can customize the behavior of this script through a global `coi` variable.
        const coi = {
            shouldRegister: () => !reloadedBySelf,
            shouldDeregister: () => false,
            coepCredentialless: () => true,
            coepDegrade: () => true,
            doReload: () => window.location.reload(),
            quiet: false,
            ...window.coi
        };

        const n = navigator;
        const controlling = n.serviceWorker && n.serviceWorker.controller;

        // Record the failure if the page is served by serviceWorker.
        if (controlling && !window.crossOriginIsolated) {
            window.sessionStorage.setItem("coiCoepHasFailed", "true");
        }
        const coepHasFailed = window.sessionStorage.getItem("coiCoepHasFailed");

        if (controlling) {
            // Reload only on the first failure.
            const reloadToDegrade = coi.coepDegrade() && !(
                coepDegrading || window.crossOriginIsolated
            );
            n.serviceWorker.controller.postMessage({
                type: "coepCredentialless",
                value: (reloadToDegrade || coepHasFailed && coi.coepDegrade())
                    ? false
                    : coi.coepCredentialless(),
            });
            if (reloadToDegrade) {
                !coi.quiet && console.log("Reloading page to degrade COEP.");
                window.sessionStorage.setItem("coiReloadedBySelf", "coepdegrade");
                coi.doReload("coepdegrade");
            }

            if (coi.shouldDeregister()) {
                n.serviceWorker.controller.postMessage({ type: "deregister" });
            }
        }

        // If we're already coi: do nothing. Perhaps it's due to this script doing its job, or COOP/COEP are
        // already set from the origin server. Also if the browser has no notion of crossOriginIsolated, just give up here.
        if (window.crossOriginIsolated !== false || !coi.shouldRegister()) {
            // Reload once to set the COEP for this service worker as well
            if (!window.sessionStorage.getItem("coiReloadedAfterSuccess")) {
                !coi.quiet && console.log("Reloading page to set COEP for this service worker.");
                window.sessionStorage.setItem("coiReloadedAfterSuccess", "true");
                coi.doReload("coepaftersuccess");
            } else {
                window.sessionStorage.removeItem("luminolCheckedForUpdate");
                window.sessionStorage.removeItem("coiReloadedAfterSuccess");
            }
            return;
        }
        window.sessionStorage.removeItem("coiReloadedAfterSuccess");

        if (!window.isSecureContext) {
            !coi.quiet && console.log("COOP/COEP Service Worker not registered, a secure context is required.");
            window.sessionStorage.removeItem("luminolCheckedForUpdate");
            return;
        }

        // In some environments (e.g. Firefox private mode) this won't be available
        if (!n.serviceWorker) {
            !coi.quiet && console.error("COOP/COEP Service Worker not registered, perhaps due to private mode.");
            window.sessionStorage.removeItem("luminolCheckedForUpdate");
            return;
        }

        n.serviceWorker.register(window.document.currentScript.src).then(
            (registration) => {
                !coi.quiet && console.log("COOP/COEP Service Worker registered", registration.scope);

                registration.addEventListener("updatefound", () => {
                    !coi.quiet && console.log("Reloading page to make use of updated COOP/COEP Service Worker.");
                    window.sessionStorage.setItem("coiReloadedBySelf", "updatefound");
                    coi.doReload();
                });

                // If the registration is active, but it's not controlling the page
                if (registration.active && !n.serviceWorker.controller) {
                    !coi.quiet && console.log("Reloading page to make use of COOP/COEP Service Worker.");
                    window.sessionStorage.setItem("coiReloadedBySelf", "notcontrolling");
                    coi.doReload();
                }
            },
            (err) => {
                window.sessionStorage.removeItem("luminolCheckedForUpdate");
                !coi.quiet && console.error("COOP/COEP Service Worker failed to register:", err);
            }
        );
    })();
}
