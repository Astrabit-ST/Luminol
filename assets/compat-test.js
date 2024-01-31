// Copyright (C) 2023 Lily Lyons
//
// This file is part of Luminol.
//
// Luminol is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Luminol is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Luminol.  If not, see <http://www.gnu.org/licenses/>.

function error(msg) {
    console.error(msg);
    alert(msg);
    window.stop();
    throw null;
}

// We need a secure context for several things:
//  * File System API
//  * Service Workers
//  * WebGPU
if (!window.isSecureContext) {
    error(
        "Luminol does not work with http://. "
        + "Please visit the https:// version of this website."
    );
}

// Firefox in private browsing mode doesn't work because service workers aren't
// available and we need service workers
if (window.netscape !== undefined && !window.navigator.serviceWorker) {
    error(
        "Due to Firefox bug #1320796, you cannot use Luminol in private "
        + "browsing mode in Firefox, Tor Browser, IceCat or any other "
        + "Firefox-based browser.\n\nPlease exit private browsing mode or use "
        + "a different browser."
    );
}
