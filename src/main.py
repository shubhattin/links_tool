#!/usr/bin/env python3

from fastapi import FastAPI
from fastapi.responses import RedirectResponse
from kry import DEV_ENV, Base
from typing import Union

APP_NAME = "Links"
if DEV_ENV:
    app = FastAPI(debug=True, title=APP_NAME)
else:
    app = FastAPI(openapi_url=None, redoc_url=None, title=APP_NAME)


@app.get("/list")
def get_links_list():
    """Redirect back to deta page"""
    return RedirectResponse(Base("others").get("deta_page")["value"])


ERR_MSG = dict(detail="Link Not Found")


RelU = lambda x: 0 if x < 0 else x


def get_redirect_response(link_obj, num=0):
    link: str = link_obj["link"]
    if "enabled" in link_obj:
        enabled = link_obj["enabled"]
        if not enabled:
            return dict(detail="Link Disabled")
    if "prefix_zeros" in link_obj:
        prefix_zeros: int = link_obj["prefix_zeros"]
        link = link.format(RelU(prefix_zeros - len(str(int(num)))) * "0" + str(num))
    print(link)
    return RedirectResponse(link)


@app.get("/{name}/{num}")
def redirect_numbered_link(name: str, num: Union[int, float]):
    """Redirecting Numbered Links"""
    link_obj = Base("links").get(name)
    if not link_obj:
        return ERR_MSG
    return get_redirect_response(link_obj, num)


@app.get("/{name}")
def redirect_link(name: str):
    """Redirecting  Links"""
    link_obj = Base("links").get(name)
    if not link_obj:
        return ERR_MSG
    if "{0}" in link_obj["link"]:
        return ERR_MSG
    return get_redirect_response(link_obj)


if DEV_ENV:
    import uvicorn

    if __name__ == "__main__":
        uvicorn.run("main:app", host="0.0.0.0", port=3030, reload=True)
