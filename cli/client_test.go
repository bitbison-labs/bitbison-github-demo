package main

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestAddSendsTheURLAndReadsTheLink(t *testing.T) {
	api := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		var body map[string]string
		if err := json.NewDecoder(r.Body).Decode(&body); err != nil {
			t.Fatal(err)
		}
		w.WriteHeader(http.StatusCreated)
		json.NewEncoder(w).Encode(Link{Slug: "0", URL: body["url"]})
	}))
	defer api.Close()

	link, err := (&Client{Base: api.URL}).Add("https://example.com/a")
	if err != nil {
		t.Fatal(err)
	}
	if link.Slug != "0" || link.URL != "https://example.com/a" {
		t.Fatalf("got %+v", link)
	}
}

func TestARefusalIsTheAPIsOwnError(t *testing.T) {
	api := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusUnprocessableEntity)
		w.Write([]byte(`{"error":"url must start with http:// or https://"}`))
	}))
	defer api.Close()

	_, err := (&Client{Base: api.URL}).Add("ftp://example.com")
	if err == nil || err.Error() != "url must start with http:// or https://" {
		t.Fatalf("got %v", err)
	}
}
