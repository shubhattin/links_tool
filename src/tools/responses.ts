export const JSONResponse = (data: any) => {
  return new Response(JSON.stringify(data), {
    headers: {
      'content-type': 'application/json'
    }
  });
};
